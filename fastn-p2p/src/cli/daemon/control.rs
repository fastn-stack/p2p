//! Control socket server for handling client requests
//!
//! This module handles the Unix domain socket that clients connect to.
//! It parses JSON requests and coordinates with the P2P layer.

use std::path::PathBuf;
use tokio::sync::broadcast;
use tokio::net::UnixListener;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use serde::{Deserialize, Serialize};

use super::{DaemonCommand, DaemonResponse};

/// Client request types - precise typing for each operation
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum ClientRequest {
    #[serde(rename = "call")]
    Call {
        from_identity: String,
        to_peer: fastn_id52::PublicKey,
        protocol: String,
        bind_alias: String,
        request: serde_json::Value,
    },
    #[serde(rename = "stream")]
    Stream {
        from_identity: String,
        to_peer: fastn_id52::PublicKey,
        protocol: String,
        bind_alias: String,
        initial_data: serde_json::Value,
    },
    #[serde(rename = "reload-identities")]
    ReloadIdentities,
    #[serde(rename = "set-identity-state")]
    SetIdentityState {
        identity: String,
        online: bool,
    },
    #[serde(rename = "add-protocol")]
    AddProtocol {
        identity: String,
        protocol: String,
        bind_alias: String,
        config: serde_json::Value,
    },
    #[serde(rename = "remove-protocol")]
    RemoveProtocol {
        identity: String,
        protocol: String,
        bind_alias: String,
    },
}

/// JSON response format to clients
#[derive(Debug, Serialize)]
struct ClientResponse {
    /// Success status: true for ok, false for error
    success: bool,
    /// Response data or error message
    data: serde_json::Value,
}

/// Run the control socket server
pub async fn run(
    fastn_home: PathBuf,
    command_tx: broadcast::Sender<DaemonCommand>,
    mut response_rx: broadcast::Receiver<DaemonResponse>,
) -> Result<(), Box<dyn std::error::Error>> {
    let socket_path = fastn_home.join("control.sock");
    
    // Remove existing socket if it exists
    if socket_path.exists() {
        tokio::fs::remove_file(&socket_path).await?;
    }

    let listener = UnixListener::bind(&socket_path)?;
    println!("🎧 Control socket listening on: {}", socket_path.display());

    // Start response dispatcher task to handle P2P responses
    let _response_task = tokio::spawn(async move {
        while let Ok(response) = response_rx.recv().await {
            todo!("Route response back to appropriate client connection based on request ID");
        }
    });

    loop {
        match listener.accept().await {
            Ok((stream, _addr)) => {
                let fastn_home_clone = fastn_home.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_client(stream, fastn_home_clone).await {
                        eprintln!("Error handling client: {}", e);
                    }
                });
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
    }
}

async fn handle_client(
    stream: tokio::net::UnixStream,
    fastn_home: PathBuf,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("📨 Client connected to control socket");
    
    let (reader, writer) = stream.into_split();
    let mut buf_reader = BufReader::new(reader);
    let mut line = String::new();

    // Read the first line to get request header and determine routing
    match buf_reader.read_line(&mut line).await {
        Ok(0) => {
            println!("📤 Client disconnected immediately");
            return Ok(());
        }
        Ok(_) => {
            let request_json = line.trim();
            if request_json.is_empty() {
                return Ok(());
            }

            println!("📥 Client request: {}", request_json);

            // Parse request header to determine routing strategy
            match route_client_request(&fastn_home, request_json, buf_reader, writer).await {
                Ok(_) => println!("✅ Request handled successfully"),
                Err(e) => eprintln!("❌ Request failed: {}", e),
            }
        }
        Err(e) => {
            eprintln!("Error reading client request: {}", e);
        }
    }

    Ok(())
}

/// Route client request based on type: P2P (call/stream) or control (daemon management)
async fn route_client_request(
    fastn_home: &PathBuf,
    request_json: &str,
    unix_reader: BufReader<tokio::net::unix::OwnedReadHalf>,
    unix_writer: tokio::net::unix::OwnedWriteHalf,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Parse the client request to determine routing
    let request: ClientRequest = serde_json::from_str(request_json)?;
    
    match request {
        ClientRequest::Call { from_identity, to_peer, protocol, bind_alias, request } => {
            println!("🔀 Routing P2P call: {} {} from {} to {}", 
                    protocol, bind_alias, from_identity, to_peer.id52());
            
            // P2P call routing using fastn_net connection pooling
            handle_p2p_call(fastn_home.clone(), from_identity, to_peer, protocol, bind_alias, request, unix_writer).await
        }
        ClientRequest::Stream { from_identity, to_peer, protocol, bind_alias, initial_data } => {
            println!("🔀 Routing P2P stream: {} {} from {} to {}", 
                    protocol, bind_alias, from_identity, to_peer.id52());
            
            // P2P streaming routing with bidirectional piping
            handle_p2p_stream(from_identity, to_peer, protocol, bind_alias, initial_data, unix_reader, unix_writer).await
        }
        // Control commands (non-P2P)
        ClientRequest::ReloadIdentities => {
            println!("🔀 Routing control: reload identities");
            handle_control_command("reload-identities", serde_json::Value::Null, unix_writer).await
        }
        ClientRequest::SetIdentityState { identity, online } => {
            println!("🔀 Routing control: set {} {}", identity, if online { "online" } else { "offline" });
            let data = serde_json::json!({ "identity": identity, "online": online });
            handle_control_command("set-identity-state", data, unix_writer).await
        }
        ClientRequest::AddProtocol { identity, protocol, bind_alias, config } => {
            println!("🔀 Routing control: add protocol {} {} to {}", protocol, bind_alias, identity);
            let data = serde_json::json!({ "identity": identity, "protocol": protocol, "bind_alias": bind_alias, "config": config });
            handle_control_command("add-protocol", data, unix_writer).await
        }
        ClientRequest::RemoveProtocol { identity, protocol, bind_alias } => {
            println!("🔀 Routing control: remove protocol {} {} from {}", protocol, bind_alias, identity);
            let data = serde_json::json!({ "identity": identity, "protocol": protocol, "bind_alias": bind_alias });
            handle_control_command("remove-protocol", data, unix_writer).await
        }
    }
}

/// Handle P2P call request - use fastn_net::get_stream() for connection pooling
async fn handle_p2p_call(
    fastn_home: PathBuf,
    from_identity: String,
    to_peer: fastn_id52::PublicKey,
    protocol: String,
    bind_alias: String,
    request: serde_json::Value,
    mut unix_writer: tokio::net::unix::OwnedWriteHalf,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("📞 P2P call: {} {} from {} to {}", protocol, bind_alias, from_identity, to_peer.id52());
    
    // Load real identity private key from daemon identity management
    let from_key = match load_identity_key(&fastn_home, &from_identity).await {
        Ok(key) => {
            println!("🔑 Loaded identity key for: {}", from_identity);
            key
        }
        Err(e) => {
            println!("❌ Failed to load identity '{}': {}", from_identity, e);
            let error_response = ClientResponse {
                success: false,
                data: serde_json::json!({
                    "error": format!("Identity '{}' not found or offline: {}", from_identity, e)
                }),
            };
            let response_json = serde_json::to_string(&error_response)?;
            unix_writer.write_all(response_json.as_bytes()).await?;
            return Ok(());
        }
    };
    
    // Create endpoint for this identity
    println!("🔌 Creating P2P endpoint for identity: {}", from_key.public_key().id52());
    let endpoint = fastn_net::get_endpoint(from_key).await?;
    
    // Create protocol header - for now use a basic protocol like Ping
    // TODO: Map daemon protocol strings to fastn_net Protocol enum variants  
    let protocol_header = fastn_net::ProtocolHeader {
        protocol: fastn_net::Protocol::Ping,  // Use Ping as placeholder for daemon protocols
        extra: Some(format!("{}:{}", protocol, bind_alias)),  // Include actual protocol info in extra
    };
    
    // Use global singletons for connection pooling and graceful shutdown
    let pool = fastn_p2p::pool();
    let graceful = fastn_p2p::graceful();
    
    println!("🔌 Getting P2P stream to {} via connection pool", to_peer.id52());
    let (mut p2p_sender, mut p2p_receiver) = fastn_net::get_stream(
        endpoint, 
        protocol_header, 
        &to_peer, 
        pool, 
        graceful
    ).await?;
    
    // Send the request data to P2P
    println!("📤 Sending request to P2P: {}", request);
    let request_bytes = serde_json::to_vec(&request)?;
    use tokio::io::AsyncWriteExt;
    p2p_sender.write_all(&request_bytes).await?;
    p2p_sender.finish()?; // Remove .await - it's not async
    
    // Read response from P2P 
    let response_bytes = p2p_receiver.read_to_end(1024 * 1024).await?; // 1MB limit
    let response_str = String::from_utf8_lossy(&response_bytes);
    
    println!("📥 Received P2P response: {} bytes", response_bytes.len());
    
    // Send response back to Unix socket client
    let response = ClientResponse {
        success: true,
        data: serde_json::json!({
            "p2p_response": response_str,
            "protocol": protocol,
            "bind_alias": bind_alias,
            "from_identity": from_identity
        }),
    };
    
    let response_json = serde_json::to_string(&response)?;
    unix_writer.write_all(response_json.as_bytes()).await?;
    unix_writer.write_all(b"\n").await?;
    
    println!("✅ P2P call completed and response sent to client");
    Ok(())
}

/// Handle P2P streaming request - bidirectional piping
async fn handle_p2p_stream(
    _from_identity: String,
    _to_peer: fastn_id52::PublicKey,
    _protocol: String,
    _bind_alias: String,
    _initial_data: serde_json::Value,
    _unix_reader: BufReader<tokio::net::unix::OwnedReadHalf>,
    _unix_writer: tokio::net::unix::OwnedWriteHalf,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    todo!("Use fastn_net::get_stream() for P2P connection, pipe Unix socket ↔ P2P stream bidirectionally");
}

/// Handle control commands (daemon management, non-P2P)
async fn handle_control_command(
    _command: &str,
    _data: serde_json::Value,
    _unix_writer: tokio::net::unix::OwnedWriteHalf,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    todo!("Handle daemon management commands: reload identities, add/remove protocols, set online/offline");
}

/// Load identity private key from daemon identity management
async fn load_identity_key(
    fastn_home: &PathBuf,
    identity_name: &str,
) -> Result<fastn_id52::SecretKey, Box<dyn std::error::Error + Send + Sync>> {
    let identities_dir = fastn_home.join("identities");
    let identity_dir = identities_dir.join(identity_name);
    
    // Check if identity exists
    if !identity_dir.exists() {
        return Err(format!("Identity '{}' not found in {}", identity_name, identity_dir.display()).into());
    }
    
    // Check if identity is online
    let online_marker = identity_dir.join("online");
    if !online_marker.exists() {
        return Err(format!("Identity '{}' is offline", identity_name).into());
    }
    
    // Load the identity private key
    match fastn_id52::SecretKey::load_from_dir(&identity_dir, "identity") {
        Ok((_id52, secret_key)) => {
            println!("🔑 Loaded key for identity '{}': {}", identity_name, secret_key.public_key().id52());
            Ok(secret_key)
        }
        Err(e) => {
            Err(format!("Failed to load key for identity '{}': {}", identity_name, e).into())
        }
    }
}