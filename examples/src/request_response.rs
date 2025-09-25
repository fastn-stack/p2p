//! Request/Response Pattern Example (Client-Only)
//!
//! Demonstrates the lightweight fastn-p2p-client making requests via daemon.
//! Server logic is now implemented as a protocol in the fastn-p2p daemon.
//!
//! Usage:
//!   1. Start daemon: fastn-p2p daemon
//!   2. Configure Echo protocol on an identity in the daemon  
//!   3. Run client: cargo run --bin request_response <peer_id52> [message]

use fastn_p2p_client as fastn_p2p;

// Import protocol types from the daemon (for client usage)
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum EchoProtocol {
    Echo,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EchoRequest {
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EchoResponse {
    pub echoed: String,
}

#[derive(Serialize, Deserialize, Debug, thiserror::Error)]
pub enum EchoError {
    #[error("Invalid message: {0}")]
    InvalidMessage(String),
}

type EchoResult = Result<EchoResponse, EchoError>;

#[fastn_p2p::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <peer_id52> [message]", args[0]);
        eprintln!("");
        eprintln!("This client connects to a fastn-p2p daemon and sends Echo requests.");
        eprintln!("Make sure the daemon is running with an Echo protocol configured:");
        eprintln!("  1. fastn-p2p daemon");
        eprintln!("  2. fastn-p2p create-identity alice");
        eprintln!("  3. fastn-p2p add-protocol alice --protocol Echo --config '{{\"max_message_length\": 1000}}'");
        return Ok(());
    }
    
    let target_id52 = &args[1];
    let message = args.get(2).unwrap_or(&"Hello P2P via daemon!".to_string()).clone();
    
    run_client(target_id52, message).await
}

async fn run_client(
    target_id52: &str,
    message: String,
) -> Result<(), Box<dyn std::error::Error>> {
    // Parse target peer ID
    let target: fastn_p2p::PublicKey = target_id52.parse()
        .map_err(|e| format!("Invalid peer ID '{}': {}", target_id52, e))?;
    
    // Generate ephemeral client key (daemon manages the actual identity keys)
    let private_key = fastn_p2p::SecretKey::generate();

    println!("📤 Sending '{}' to {} via daemon", message, target.id52());

    let request = EchoRequest { message };
    
    // Use lightweight client that routes through daemon
    let result: EchoResult = fastn_p2p::call(private_key, target, EchoProtocol::Echo, request).await?;

    match result {
        Ok(response) => println!("✅ Response: {}", response.echoed),
        Err(error) => println!("❌ Error: {:?}", error),
    }
    
    Ok(())
}
