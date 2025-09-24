//! Daemon functionality for fastn-p2p
//!
//! The daemon runs two main services:
//! 1. Control socket server - handles client requests via Unix domain socket
//! 2. P2P listener - handles incoming P2P connections and protocols

use std::path::PathBuf;
use tokio::sync::broadcast;

pub mod control;
pub mod p2p;
pub mod protocols;
pub mod test_protocols;

/// Daemon command for coordinating between control socket and P2P
#[derive(Debug, Clone)]
pub enum DaemonCommand {
    /// Make a request/response call to a peer
    Call {
        peer: fastn_id52::PublicKey,
        protocol: String,
        request_data: serde_json::Value,
    },
    /// Open a stream to a peer  
    Stream {
        peer: fastn_id52::PublicKey,
        protocol: String,
        initial_data: serde_json::Value,
    },
    /// Shutdown the daemon
    Shutdown,
}

/// Daemon response back to control socket clients
#[derive(Debug, Clone)]
pub enum DaemonResponse {
    /// Successful call response
    CallResponse {
        response_data: serde_json::Value,
    },
    /// Call error
    CallError {
        error: String,
    },
    /// Stream established
    StreamReady {
        stream_id: u64,
    },
    /// Stream error
    StreamError {
        error: String,
    },
}

/// Run the fastn-p2p daemon with both control socket and P2P listener
pub async fn run(fastn_home: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    // Ensure FASTN_HOME directory exists
    tokio::fs::create_dir_all(&fastn_home).await?;

    // Generate or load daemon private key
    let daemon_key = get_or_create_daemon_key(&fastn_home).await?;
    println!("🔑 Daemon peer ID: {}", daemon_key.public_key().id52());

    // Create communication channels between control socket and P2P
    let (command_tx, _command_rx) = broadcast::channel::<DaemonCommand>(100);
    let (response_tx, _response_rx) = broadcast::channel::<DaemonResponse>(100);

    // Start P2P listener in background
    let p2p_task = {
        let daemon_key = daemon_key.clone();
        let command_rx = command_tx.subscribe();
        let response_tx = response_tx.clone();
        tokio::spawn(async move {
            if let Err(e) = p2p::run(daemon_key, command_rx, response_tx).await {
                eprintln!("P2P listener error: {}", e);
            }
        })
    };

    // Start control socket server in foreground
    let control_task = {
        let command_tx = command_tx.clone();
        let response_rx = response_tx.subscribe();
        tokio::spawn(async move {
            if let Err(e) = control::run(fastn_home, command_tx, response_rx).await {
                eprintln!("Control socket error: {}", e);
            }
        })
    };

    println!("🚀 Daemon started successfully");
    println!("   - P2P listener active");
    println!("   - Control socket ready");

    // Wait for either task to complete (shouldn't happen in normal operation)
    tokio::select! {
        _ = p2p_task => {
            println!("P2P listener shut down");
        }
        _ = control_task => {
            println!("Control socket shut down");
        }
    }

    Ok(())
}

async fn get_or_create_daemon_key(
    fastn_home: &PathBuf,
) -> Result<fastn_id52::SecretKey, Box<dyn std::error::Error>> {
    let key_file = fastn_home.join("daemon.key");

    // Try to load existing key
    if key_file.exists() {
        if let Ok(key_bytes) = tokio::fs::read(&key_file).await {
            if key_bytes.len() == 32 {
                let mut bytes_array = [0u8; 32];
                bytes_array.copy_from_slice(&key_bytes);
                let secret_key = fastn_id52::SecretKey::from_bytes(&bytes_array);
                println!("🔑 Loaded daemon key from: {}", key_file.display());
                return Ok(secret_key);
            }
        }
        println!("⚠️  Could not load key from {}, generating new one", key_file.display());
    }

    // Generate new key
    let key = fastn_id52::SecretKey::generate();

    // Try to save it
    match tokio::fs::write(&key_file, &key.to_secret_bytes()).await {
        Ok(_) => {
            println!("🔑 Generated and saved daemon key to: {}", key_file.display());
        }
        Err(e) => {
            println!("⚠️  Could not save key to {} ({})", key_file.display(), e);
            println!("   Using temporary key - daemon ID will change on restart");
        }
    }

    Ok(key)
}