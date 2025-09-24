//! P2P networking layer for the daemon
//!
//! This module handles incoming P2P connections using iroh and routes
//! them to appropriate protocol handlers.

use tokio::sync::broadcast;
use std::collections::HashMap;

use super::{DaemonCommand, DaemonResponse};
use super::protocols::{echo, shell};

/// P2P listener that handles incoming connections and protocol routing
pub async fn run(
    daemon_key: fastn_id52::SecretKey,
    mut command_rx: broadcast::Receiver<DaemonCommand>,
    response_tx: broadcast::Sender<DaemonResponse>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🌐 Starting P2P listener...");

    // Initialize P2P endpoint with daemon's private key
    let _endpoint = fastn_net::get_endpoint(daemon_key.clone()).await?;
    println!("🎧 P2P listening on peer ID: {}", daemon_key.public_key().id52());

    // Start protocol handlers
    let protocol_handlers = setup_protocol_handlers(daemon_key.clone(), response_tx.clone()).await?;
    println!("📡 Protocol handlers active: {}", 
             protocol_handlers.keys().map(|k| k.as_str()).collect::<Vec<_>>().join(", "));

    // Command processing loop
    tokio::spawn(async move {
        while let Ok(command) = command_rx.recv().await {
            match command {
                DaemonCommand::Call { peer, protocol, request_data } => {
                    println!("📞 Outgoing call to {} for protocol {}", peer.id52(), protocol);
                    
                    // TODO: Make actual P2P call using fastn_p2p::client::call
                    // For now, simulate a response
                    let response = DaemonResponse::CallResponse {
                        response_data: serde_json::json!({
                            "simulated": true,
                            "original_request": request_data,
                            "protocol": protocol,
                            "peer": peer.id52()
                        }),
                    };
                    
                    if let Err(e) = response_tx.send(response) {
                        eprintln!("Failed to send call response: {}", e);
                    }
                }
                DaemonCommand::Stream { peer, protocol, initial_data: _ } => {
                    println!("🌊 Outgoing stream to {} for protocol {}", peer.id52(), protocol);
                    
                    // TODO: Make actual P2P stream using fastn_p2p::client::connect
                    let response = DaemonResponse::StreamReady {
                        stream_id: rand::random::<u64>(), // TODO: Real stream ID
                    };
                    
                    if let Err(e) = response_tx.send(response) {
                        eprintln!("Failed to send stream response: {}", e);
                    }
                }
                DaemonCommand::Shutdown => {
                    println!("🛑 P2P listener shutting down");
                    break;
                }
            }
        }
    });

    // Keep the endpoint alive
    // TODO: Actually listen for incoming P2P connections using fastn_p2p::listen
    println!("🎯 P2P listener ready for connections");
    
    // For now, just keep running
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        // TODO: Replace with actual P2P event loop
    }
}

async fn setup_protocol_handlers(
    daemon_key: fastn_id52::SecretKey,
    response_tx: broadcast::Sender<DaemonResponse>,
) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    let mut handlers = HashMap::new();
    
    // Initialize Echo protocol handler
    echo::initialize(daemon_key.clone(), response_tx.clone()).await?;
    handlers.insert(super::test_protocols::ECHO_PROTOCOL.to_string(), "Active".to_string());
    
    // Initialize Shell protocol handler  
    shell::initialize(daemon_key.clone(), response_tx.clone()).await?;
    handlers.insert(super::test_protocols::SHELL_PROTOCOL.to_string(), "Active".to_string());
    
    Ok(handlers)
}