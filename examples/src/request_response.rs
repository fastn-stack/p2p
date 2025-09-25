//! Request/Response Pattern Example (Server-Only)
//!
//! Pure Echo protocol server using fastn-p2p server APIs.
//! Clients use fastn-p2p CLI commands to test this server.
//!
//! Usage:
//!   Server: cargo run --bin request_response [identity_name]
//!   Client: echo '{"message":"Hello"}' | fastn-p2p call <server_peer_id> Echo

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

#[fastn_p2p::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    
    let identity = args.get(1).unwrap_or(&"alice".to_string()).clone();
    
    println!("🎧 Starting Echo protocol server for identity: {}", identity);
    println!("📡 Testing setup:");
    println!("  1. Make sure daemon is running: fastn-p2p daemon");
    println!("  2. Create identity: fastn-p2p create-identity {}", identity);
    println!("  3. Add protocol: fastn-p2p add-protocol {} --protocol Echo --config '{{\"max_length\": 1000}}'", identity);
    println!("  4. Set online: fastn-p2p identity-online {}", identity);
    println!("  5. Test with CLI: echo '{{\"message\":\"Hello\"}}' | fastn-p2p call <peer_id> Echo");
    println!("");
    
    run_server(identity).await
}

async fn run_server(_identity: String) -> Result<(), Box<dyn std::error::Error>> {
    println!("🎧 Starting multi-identity Echo protocol server");
    println!("📡 Will discover and serve all configured identities and protocols from FASTN_HOME");
    
    // Use modern serve_all() builder that discovers all identities and protocols
    fastn_p2p::serve_all()
        .handle_requests("Echo", fastn_p2p::echo_request_handler)
        .serve()
        .await?;
    
    Ok(())
}

/// Load identity private key from current environment
async fn load_identity_key(identity: &str) -> Result<fastn_p2p::SecretKey, Box<dyn std::error::Error>> {
    // Use FASTN_HOME environment variable to locate identity
    let fastn_home = std::env::var("FASTN_HOME")
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or("/tmp".to_string());
            format!("{}/.fastn", home)
        });
    
    let identity_dir = std::path::PathBuf::from(fastn_home)
        .join("identities")
        .join(identity);
    
    // Load the private key using fastn-id52
    match fastn_p2p::SecretKey::load_from_dir(&identity_dir, "identity") {
        Ok((_id52, secret_key)) => Ok(secret_key),
        Err(e) => Err(format!("Failed to load identity '{}': {}", identity, e).into()),
    }
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
