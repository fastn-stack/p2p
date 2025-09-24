//! Daemon functionality for fastn-p2p

use std::path::PathBuf;
use tokio::net::UnixListener;

/// Run the fastn-p2p daemon
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