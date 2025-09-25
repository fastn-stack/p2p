//! Request/Response Pattern Example (Client + Server)
//!
//! Demonstrates both client and server sides of the Echo protocol.
//! Can run as either client (via lightweight fastn-p2p-client) or server (using fastn-p2p server APIs).
//!
//! Usage:
//!   Server: cargo run --bin request_response server [identity_name]
//!   Client: cargo run --bin request_response client <peer_id52> [message]

use fastn_p2p_client;
use serde::{Serialize, Deserialize};

// Echo Protocol Definition
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

#[fastn_p2p_client::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage:");
        eprintln!("  {} server [identity_name]           # Start Echo protocol server", args[0]);
        eprintln!("  {} client <peer_id52> [message]     # Send Echo request to server", args[0]);
        eprintln!("");
        eprintln!("Two-daemon testing setup:");
        eprintln!("  1. Terminal 1: FASTN_HOME=/tmp/alice fastn-p2p daemon");
        eprintln!("  2. Terminal 2: FASTN_HOME=/tmp/bob fastn-p2p daemon");  
        eprintln!("  3. Terminal 3: FASTN_HOME=/tmp/alice {} server alice", args[0]);
        eprintln!("  4. Terminal 4: FASTN_HOME=/tmp/bob {} client <alice_id52> \"Hello!\"", args[0]);
        return Ok(());
    }
    
    match args[1].as_str() {
        "server" => {
            let identity = args.get(2).unwrap_or(&"alice".to_string()).clone();
            run_server(identity).await
        }
        "client" => {
            if args.len() < 3 {
                return Err("Client mode requires peer_id52 argument".into());
            }
            let target_id52 = &args[2];
            let message = args.get(3).unwrap_or(&"Hello P2P via daemon!".to_string()).clone();
            run_client(target_id52, message).await
        }
        _ => {
            return Err("First argument must be 'server' or 'client'".into());
        }
    }
}

async fn run_server(identity: String) -> Result<(), Box<dyn std::error::Error>> {
    println!("🎧 Starting Echo protocol server for identity: {}", identity);
    println!("📡 Server will handle Echo requests via fastn-p2p server APIs");
    
    // TODO: Use fastn_p2p server APIs to listen for Echo protocol requests
    // This will use the clean server APIs from fastn-p2p crate
    todo!("Implement Echo protocol server using fastn_p2p::listen() or similar server API");
}

async fn run_client(
    target_id52: &str,
    message: String,
) -> Result<(), Box<dyn std::error::Error>> {
    // Parse target peer ID to PublicKey for type safety
    let target_peer: fastn_p2p_client::PublicKey = target_id52.parse()
        .map_err(|e| format!("Invalid peer ID '{}': {}", target_id52, e))?;
        
    println!("📤 Sending '{}' to {} via daemon", message, target_peer.id52());

    let request = EchoRequest { message };
    
    // Use lightweight client that routes through daemon
    let result: EchoResult = fastn_p2p_client::call(
        "bob",                // From bob identity (daemon manages keys)
        target_peer,          // To target peer (alice)
        "Echo",               // Echo protocol
        "default",            // Default Echo instance
        request               // Request data
    ).await?;

    match result {
        Ok(response) => println!("✅ Response: {}", response.echoed),
        Err(error) => println!("❌ Error: {:?}", error),
    }
    
    Ok(())
}

/// Echo request handler (server-side logic)
pub async fn echo_handler(req: EchoRequest) -> Result<EchoResponse, EchoError> {
    println!("💬 Server received: {}", req.message);
    
    // Basic validation
    if req.message.is_empty() {
        return Err(EchoError::InvalidMessage("Message cannot be empty".to_string()));
    }
    
    let response = EchoResponse {
        echoed: format!("Echo from server: {}", req.message),
    };
    
    println!("📤 Server responding: {}", response.echoed);
    Ok(response)
}
