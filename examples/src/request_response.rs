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
    
    if args.len() < 2 {
        eprintln!("Usage:");
        eprintln!("  {} init                             # Initialize FASTN_HOME environment", args[0]);
        eprintln!("  {} run                              # Start Echo protocol server", args[0]);
        eprintln!("");
        eprintln!("Testing sequence:");
        eprintln!("  1. FASTN_HOME=/tmp/alice {} init", args[0]);
        eprintln!("  2. FASTN_HOME=/tmp/alice fastn-p2p daemon &");
        eprintln!("  3. FASTN_HOME=/tmp/alice fastn-p2p create-identity alice");
        eprintln!("  4. FASTN_HOME=/tmp/alice fastn-p2p add-protocol alice --protocol echo.fastn.com");
        eprintln!("  5. FASTN_HOME=/tmp/alice fastn-p2p identity-online alice");
        eprintln!("  6. FASTN_HOME=/tmp/alice {} run", args[0]);
        eprintln!("  7. echo '{{\"message\":\"Hello\"}}' | FASTN_HOME=/tmp/alice fastn-p2p call <alice_peer_id> echo.fastn.com basic-echo");
        return Ok(());
    }
    
    match args[1].as_str() {
        "init" => {
            init_environment().await
        }
        "run" => {
            run_server().await
        }
        _ => {
            return Err("Command must be 'init' or 'run'".into());
        }
    }
}

async fn init_environment() -> Result<(), Box<dyn std::error::Error>> {
    let fastn_home = std::env::var("FASTN_HOME")
        .unwrap_or_else(|_| "/tmp/alice".to_string());
    
    println!("🔧 Initializing FASTN_HOME: {}", fastn_home);
    
    // Create basic directory structure
    let fastn_path = std::path::PathBuf::from(&fastn_home);
    tokio::fs::create_dir_all(&fastn_path).await?;
    tokio::fs::create_dir_all(fastn_path.join("identities")).await?;
    
    println!("✅ FASTN_HOME initialized");
    println!("📁 Directory structure created");
    println!("");
    println!("Next steps:");
    println!("  1. Start daemon: FASTN_HOME={} fastn-p2p daemon &", fastn_home);
    println!("  2. Create identity: FASTN_HOME={} fastn-p2p create-identity alice", fastn_home);
    println!("  3. Add protocol: FASTN_HOME={} fastn-p2p add-protocol alice --protocol echo.fastn.com --config '{{\"max_length\": 1000}}'", fastn_home);
    println!("  4. Set online: FASTN_HOME={} fastn-p2p identity-online alice", fastn_home);
    println!("  5. Start server: FASTN_HOME={} {} run", fastn_home, std::env::args().collect::<Vec<_>>()[0]);
    
    Ok(())
}

async fn run_server() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎧 Starting multi-identity Echo protocol server");
    println!("📡 Will discover and serve all configured identities and protocols from FASTN_HOME");
    
    // Use modern serve_all() builder with complete lifecycle
    fastn_p2p::serve_all()
        .protocol("echo.fastn.com", |p| p
            .on_init(echo_init_handler)
            .on_load(echo_load_handler)
            .on_check(echo_check_handler)
            .handle_requests("basic-echo", fastn_p2p::echo_request_handler)
            .on_reload(echo_reload_handler)
            .on_stop(echo_stop_handler)
        )
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

// Echo protocol lifecycle handlers
use std::pin::Pin;
use std::future::Future;

fn echo_init_handler(
    identity: &str,
    bind_alias: &str, 
    protocol_dir: &std::path::PathBuf,
) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send>> {
    let identity = identity.to_string();
    let bind_alias = bind_alias.to_string();
    let protocol_dir = protocol_dir.clone();
    
    Box::pin(async move {
        println!("🔧 Echo init: {} {} ({})", identity, bind_alias, protocol_dir.display());
        // TODO: Create default config files, setup protocol workspace
        Ok(())
    })
}

fn echo_load_handler(
    identity: &str,
    bind_alias: &str,
    protocol_dir: &std::path::PathBuf,
) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send>> {
    let identity = identity.to_string();
    let bind_alias = bind_alias.to_string();
    let protocol_dir = protocol_dir.clone();
    
    Box::pin(async move {
        println!("🚀 Echo load: {} {} ({})", identity, bind_alias, protocol_dir.display());
        // TODO: Read config, start P2P listeners, initialize runtime state
        Ok(())
    })
}

fn echo_check_handler(
    identity: &str,
    bind_alias: &str,
    protocol_dir: &std::path::PathBuf,
) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send>> {
    let identity = identity.to_string();
    let bind_alias = bind_alias.to_string();
    let protocol_dir = protocol_dir.clone();
    
    Box::pin(async move {
        println!("🔍 Echo check: {} {} ({})", identity, bind_alias, protocol_dir.display());
        // TODO: Validate config files, check runtime state
        Ok(())
    })
}

fn echo_reload_handler(
    identity: &str,
    bind_alias: &str,
    protocol_dir: &std::path::PathBuf,
) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send>> {
    let identity = identity.to_string();
    let bind_alias = bind_alias.to_string();
    let protocol_dir = protocol_dir.clone();
    
    Box::pin(async move {
        println!("🔄 Echo reload: {} {} ({})", identity, bind_alias, protocol_dir.display());
        // TODO: Re-read config, restart services with new settings
        Ok(())
    })
}

fn echo_stop_handler(
    identity: &str,
    bind_alias: &str,
    protocol_dir: &std::path::PathBuf,
) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send>> {
    let identity = identity.to_string();
    let bind_alias = bind_alias.to_string();
    let protocol_dir = protocol_dir.clone();
    
    Box::pin(async move {
        println!("🛑 Echo stop: {} {} ({})", identity, bind_alias, protocol_dir.display());
        // TODO: Clean shutdown of P2P listeners, save state
        Ok(())
    })
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
