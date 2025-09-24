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

/// JSON request format from clients
#[derive(Debug, Deserialize)]
struct ClientRequest {
    /// Request ID for tracking responses
    id: String,
    /// Request type: "call" or "stream"
    #[serde(rename = "type")]
    request_type: String,
    /// Target peer ID52
    target: String,
    /// Protocol name
    protocol: String,
    /// Request data (varies by protocol)
    data: serde_json::Value,
}

/// JSON response format to clients
#[derive(Debug, Serialize)]
struct ClientResponse {
    /// Request ID from original request
    id: String,
    /// Status: "ok" or "error"
    status: String,
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

    // TODO: Start response dispatcher task to handle P2P responses
    let _response_task = tokio::spawn(async move {
        while let Ok(response) = response_rx.recv().await {
            // TODO: Route response back to appropriate client connection
            println!("📨 Got daemon response: {:?}", response);
        }
    });

    loop {
        match listener.accept().await {
            Ok((stream, _addr)) => {
                let command_tx = command_tx.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_client(stream, command_tx).await {
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
    command_tx: broadcast::Sender<DaemonCommand>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("📨 Client connected to control socket");
    
    let (reader, mut writer) = stream.into_split();
    let mut buf_reader = BufReader::new(reader);
    let mut line = String::new();

    loop {
        line.clear();
        match buf_reader.read_line(&mut line).await {
            Ok(0) => {
                // EOF - client disconnected
                println!("📤 Client disconnected from control socket");
                break;
            }
            Ok(_) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }

                match handle_request(trimmed, &command_tx).await {
                    Ok(response) => {
                        let response_json = serde_json::to_string(&response)?;
                        writer.write_all(response_json.as_bytes()).await?;
                        writer.write_all(b"\n").await?;
                        writer.flush().await?;
                    }
                    Err(e) => {
                        let error_msg = e.to_string();
                        let error_response = ClientResponse {
                            id: "unknown".to_string(),
                            status: "error".to_string(), 
                            data: serde_json::json!({ "message": error_msg }),
                        };
                        let response_json = serde_json::to_string(&error_response)?;
                        writer.write_all(response_json.as_bytes()).await?;
                        writer.write_all(b"\n").await?;
                        writer.flush().await?;
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading from client: {}", e);
                break;
            }
        }
    }

    Ok(())
}

async fn handle_request(
    request_json: &str,
    command_tx: &broadcast::Sender<DaemonCommand>,
) -> Result<ClientResponse, Box<dyn std::error::Error + Send + Sync>> {
    let request: ClientRequest = serde_json::from_str(request_json)?;
    
    // Parse peer ID
    let peer_id: fastn_id52::PublicKey = request.target.parse()
        .map_err(|e| format!("Invalid peer ID: {}", e))?;

    println!("🔍 Processing {} request for protocol {} to peer {}", 
             request.request_type, request.protocol, peer_id.id52());

    match request.request_type.as_str() {
        "call" => {
            // Send command to P2P layer
            let command = DaemonCommand::Call {
                peer: peer_id,
                protocol: request.protocol.clone(),
                request_data: request.data,
            };
            
            if let Err(e) = command_tx.send(command) {
                return Ok(ClientResponse {
                    id: request.id,
                    status: "error".to_string(),
                    data: serde_json::json!({ "message": format!("Failed to send command: {}", e) }),
                });
            }

            // TODO: Wait for actual response from P2P layer
            // For now, return a placeholder response
            Ok(ClientResponse {
                id: request.id,
                status: "ok".to_string(),
                data: serde_json::json!({ 
                    "message": format!("Call to {} protocol {} queued", peer_id.id52(), request.protocol)
                }),
            })
        }
        "stream" => {
            // Send stream command to P2P layer
            let command = DaemonCommand::Stream {
                peer: peer_id,
                protocol: request.protocol.clone(),
                initial_data: request.data,
            };
            
            if let Err(e) = command_tx.send(command) {
                return Ok(ClientResponse {
                    id: request.id,
                    status: "error".to_string(),
                    data: serde_json::json!({ "message": format!("Failed to send command: {}", e) }),
                });
            }

            // TODO: Establish actual stream and return stream ID
            Ok(ClientResponse {
                id: request.id,
                status: "streaming".to_string(),
                data: serde_json::json!({ 
                    "message": format!("Stream to {} protocol {} initiated", peer_id.id52(), request.protocol)
                }),
            })
        }
        _ => Ok(ClientResponse {
            id: request.id,
            status: "error".to_string(),
            data: serde_json::json!({ "message": format!("Unknown request type: {}", request.request_type) }),
        }),
    }
}