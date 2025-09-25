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
        to_peer: String, // Will convert to PublicKey in next commit
        protocol: String,
        bind_alias: String,
        request: serde_json::Value,
    },
    #[serde(rename = "stream")]
    Stream {
        from_identity: String,
        to_peer: String, // Will convert to PublicKey in next commit
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

    // Start response dispatcher task to handle P2P responses
    let _response_task = tokio::spawn(async move {
        while let Ok(response) = response_rx.recv().await {
            todo!("Route response back to appropriate client connection based on request ID");
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
    _stream: tokio::net::UnixStream,
    _command_tx: broadcast::Sender<DaemonCommand>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    todo!("Parse JSON requests from Unix socket, route to P2P layer, stream responses back");
}

async fn handle_request(
    _request_json: &str,
    _command_tx: &broadcast::Sender<DaemonCommand>,
) -> Result<ClientResponse, Box<dyn std::error::Error + Send + Sync>> {
    todo!("Parse JSON request, validate peer ID, route to P2P layer, wait for response");
}