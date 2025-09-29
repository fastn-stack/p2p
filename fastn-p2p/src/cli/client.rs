//! Client functionality for fastn-p2p CLI
//!
//! This module handles CLI client commands using the lightweight fastn-p2p-client crate.
//! It provides the same functionality as the examples but via CLI interface.

use std::path::PathBuf;
use std::io::{self, Read};

/// Make a request/response call to a peer via the daemon
pub async fn call(
    fastn_home: PathBuf,
    peer_id52: String,
    protocol: String,
    bind_alias: String,
    as_identity: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Check if daemon is running
    let socket_path = fastn_home.join("control.sock");
    if !socket_path.exists() {
        return Err(format!("Daemon not running. Socket not found: {}. Start with: fastn-p2p daemon", socket_path.display()).into());
    }
    
    // Determine identity to send from
    let from_identity = match as_identity {
        Some(identity) => identity,
        None => {
            // TODO: Auto-detect identity if only one configured
            "alice".to_string() // Hardcoded for testing
        }
    };
    
    // Parse peer ID to PublicKey for type safety
    let to_peer: fastn_id52::PublicKey = peer_id52.parse()
        .map_err(|e| format!("Invalid peer ID '{}': {}", peer_id52, e))?;
    
    // Read JSON request from stdin
    let mut stdin_input = String::new();
    io::stdin().read_to_string(&mut stdin_input)?;
    let stdin_input = stdin_input.trim();
    
    if stdin_input.is_empty() {
        return Err("No JSON input provided on stdin".into());
    }
    
    // Parse JSON to validate it's valid
    let request_json: serde_json::Value = serde_json::from_str(stdin_input)?;
    
    println!("📤 Sending {} {} request from {} to {}", protocol, bind_alias, from_identity, to_peer.id52());
    
    // Connect to daemon control socket directly
    use tokio::net::UnixStream;
    use tokio::io::{AsyncWriteExt, AsyncReadExt, AsyncBufReadExt, BufReader};
    
    let mut stream = UnixStream::connect(&socket_path).await
        .map_err(|e| format!("Failed to connect to daemon: {}", e))?;
    
    // Create typed request using shared daemon protocol structure
    let daemon_request = fastn_p2p_client::DaemonRequest::Call {
        from_identity,
        to_peer,
        protocol,
        bind_alias,
        request: request_json,
    };
    
    // Send request to daemon
    let request_data = serde_json::to_string(&daemon_request)?;
    stream.write_all(request_data.as_bytes()).await?;
    stream.write_all(b"\n").await?;
    
    println!("📡 Request sent to daemon, reading response...");
    
    // Read response from daemon
    let (reader, _writer) = stream.into_split();
    let mut buf_reader = BufReader::new(reader);
    let mut response_line = String::new();
    
    match buf_reader.read_line(&mut response_line).await {
        Ok(0) => return Err("Daemon closed connection without response".into()),
        Ok(_) => {
            let response: serde_json::Value = serde_json::from_str(response_line.trim())?;
            println!("📥 Response from daemon:");
            println!("{}", serde_json::to_string_pretty(&response)?);
        }
        Err(e) => return Err(format!("Failed to read daemon response: {}", e).into()),
    }
    
    Ok(())
}

/// Make a request/response call with args support (issue #13)
pub async fn call_with_args(
    fastn_home: PathBuf,
    peer_id52: String,
    protocol: String,
    command: String,
    bind_alias: String,
    extra_args: Vec<String>,
    as_identity: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Check if daemon is running
    let socket_path = fastn_home.join("control.sock");
    if !socket_path.exists() {
        return Err(format!("Daemon not running. Socket not found: {}. Start with: request_response run", socket_path.display()).into());
    }
    
    // Determine identity to send from
    let from_identity = match as_identity {
        Some(identity) => identity,
        None => {
            // TODO: Auto-detect identity if only one configured
            "alice".to_string() // Hardcoded for testing
        }
    };
    
    // Parse peer ID to PublicKey for type safety
    let to_peer: fastn_id52::PublicKey = peer_id52.parse()
        .map_err(|e| format!("Invalid peer ID '{}': {}", peer_id52, e))?;
    
    // Read JSON request from stdin
    let mut stdin_input = String::new();
    io::stdin().read_to_string(&mut stdin_input)?;
    let stdin_input = stdin_input.trim();
    
    if stdin_input.is_empty() {
        return Err("No JSON input provided on stdin".into());
    }
    
    // Parse JSON to validate it's valid
    let request_json: serde_json::Value = serde_json::from_str(stdin_input)?;
    
    // Enhanced DaemonRequest with command, bind_alias, and args
    let daemon_request = fastn_p2p_client::DaemonRequest::CallWithArgs {
        from_identity,
        to_peer,
        protocol,
        command,
        bind_alias,
        args: extra_args,
        request: request_json,
    };
    
    println!("📤 Sending enhanced P2P request with args support");
    
    // Connect to daemon control socket directly
    use tokio::net::UnixStream;
    use tokio::io::{AsyncWriteExt, AsyncBufReadExt, BufReader};
    
    let mut stream = UnixStream::connect(&socket_path).await
        .map_err(|e| format!("Failed to connect to daemon: {}", e))?;
    
    // Send request to daemon
    let request_data = serde_json::to_string(&daemon_request)?;
    stream.write_all(request_data.as_bytes()).await?;
    stream.write_all(b"\n").await?;
    
    println!("📡 Enhanced request sent to daemon, reading response...");
    
    // Read response from daemon
    let (reader, _writer) = stream.into_split();
    let mut buf_reader = BufReader::new(reader);
    let mut response_line = String::new();
    
    match buf_reader.read_line(&mut response_line).await {
        Ok(0) => return Err("Daemon closed connection without response".into()),
        Ok(_) => {
            let response: serde_json::Value = serde_json::from_str(response_line.trim())?;
            println!("📥 Response from daemon:");
            println!("{}", serde_json::to_string_pretty(&response)?);
        }
        Err(e) => return Err(format!("Failed to read daemon response: {}", e).into()),
    }
    
    Ok(())
}

/// Open a bidirectional stream to a peer via the daemon  
pub async fn stream(
    _fastn_home: PathBuf,
    _peer: String,
    _protocol: String,
) -> Result<(), Box<dyn std::error::Error>> {
    todo!("Use fastn-p2p-client::connect() to establish stream via daemon, pipe stdin/stdout bidirectionally");
}