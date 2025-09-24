//! fastn-p2p: P2P daemon and client
//!
//! This binary provides both daemon and client functionality for P2P communication.
//! It uses Unix domain sockets for communication between client and daemon.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "fastn-p2p")]
#[command(about = "P2P daemon and client for fastn")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the P2P daemon in foreground mode
    Daemon {
        /// Custom FASTN_HOME directory (defaults to ~/.fastn)
        #[arg(long)]
        home: Option<PathBuf>,
    },
    /// Make a request/response call to a peer
    Call {
        /// Target peer ID52
        peer: String,
        /// Protocol name
        protocol: String,
        /// Custom FASTN_HOME directory (defaults to ~/.fastn)
        #[arg(long)]
        home: Option<PathBuf>,
    },
    /// Open a bidirectional stream to a peer
    Stream {
        /// Target peer ID52
        peer: String,
        /// Protocol name
        protocol: String,
        /// Custom FASTN_HOME directory (defaults to ~/.fastn)
        #[arg(long)]
        home: Option<PathBuf>,
    },
}

#[fastn_p2p::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Daemon { home } => {
            let fastn_home = get_fastn_home(home)?;
            println!("🚀 Starting fastn-p2p daemon");
            println!("📁 FASTN_HOME: {}", fastn_home.display());
            daemon::run(fastn_home).await
        }
        Commands::Call { peer, protocol, home } => {
            let fastn_home = get_fastn_home(home)?;
            client::call(fastn_home, peer, protocol).await
        }
        Commands::Stream { peer, protocol, home } => {
            let fastn_home = get_fastn_home(home)?;
            client::stream(fastn_home, peer, protocol).await
        }
    }
}

fn get_fastn_home(custom_home: Option<PathBuf>) -> Result<PathBuf, Box<dyn std::error::Error>> {
    if let Some(home) = custom_home {
        return Ok(home);
    }

    if let Ok(env_home) = std::env::var("FASTN_HOME") {
        return Ok(PathBuf::from(env_home));
    }

    let home_dir = directories::UserDirs::new()
        .ok_or("Could not determine user home directory")?
        .home_dir()
        .to_path_buf();

    Ok(home_dir.join(".fastn"))
}

mod daemon {
    use std::path::PathBuf;
    use tokio::net::UnixListener;

    pub async fn run(fastn_home: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        // Ensure FASTN_HOME directory exists
        tokio::fs::create_dir_all(&fastn_home).await?;

        let socket_path = fastn_home.join("control.sock");
        
        // Remove existing socket if it exists
        if socket_path.exists() {
            tokio::fs::remove_file(&socket_path).await?;
        }

        let listener = UnixListener::bind(&socket_path)?;
        println!("🎧 Daemon listening on: {}", socket_path.display());
        
        // TODO: Initialize P2P endpoint with persistent key
        // TODO: Start protocol handlers
        
        loop {
            match listener.accept().await {
                Ok((stream, _addr)) => {
                    tokio::spawn(async move {
                        if let Err(e) = handle_client(stream).await {
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
    ) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: Handle JSON protocol parsing
        // TODO: Route to appropriate protocol handlers
        println!("📨 Client connected");
        Ok(())
    }
}

mod client {
    use std::path::PathBuf;

    pub async fn call(
        fastn_home: PathBuf,
        peer: String,
        protocol: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let socket_path = fastn_home.join("control.sock");
        
        if !socket_path.exists() {
            eprintln!("❌ Daemon not running. Socket not found: {}", socket_path.display());
            eprintln!("   Start daemon with: fastn-p2p daemon");
            return Err("Daemon not available".into());
        }

        // TODO: Connect to daemon via Unix socket
        // TODO: Send JSON request with peer, protocol, and stdin data
        // TODO: Print response to stdout or error to stderr
        
        println!("📤 Would call {} on peer {} via daemon", protocol, peer);
        Ok(())
    }

    pub async fn stream(
        fastn_home: PathBuf,
        peer: String,
        protocol: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let socket_path = fastn_home.join("control.sock");
        
        if !socket_path.exists() {
            eprintln!("❌ Daemon not running. Socket not found: {}", socket_path.display());
            eprintln!("   Start daemon with: fastn-p2p daemon");
            return Err("Daemon not available".into());
        }

        // TODO: Connect to daemon via Unix socket
        // TODO: Send JSON stream request 
        // TODO: Pipe stdin/stdout bidirectionally
        
        println!("🌊 Would stream {} to peer {} via daemon", protocol, peer);
        Ok(())
    }
}

// Test protocols for end-to-end testing
mod protocols {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub enum TestProtocol {
        Echo,
        Shell,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct EchoRequest {
        pub message: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct EchoResponse {
        pub echoed: String,
    }

    #[derive(Debug, Serialize, Deserialize, thiserror::Error)]
    pub enum EchoError {
        #[error("Invalid message: {0}")]
        InvalidMessage(String),
    }

    // TODO: Add Shell protocol for interactive testing
}