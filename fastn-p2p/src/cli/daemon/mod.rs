//! Daemon functionality for fastn-p2p
//!
//! The daemon runs two main services:
//! 1. Control socket server - handles client requests via Unix domain socket
//! 2. P2P listener - handles incoming P2P connections and protocols

use std::path::PathBuf;
use std::fs::OpenOptions;
use fs2::FileExt;
use tokio::sync::broadcast;

/// Daemon context containing runtime state and lock
#[derive(Debug)]
pub struct DaemonContext {
    pub private_key: fastn_id52::SecretKey,
    pub peer_id: fastn_id52::PublicKey,
    pub fastn_home: PathBuf,
    pub _lock_file: std::fs::File, // Keep lock file open to maintain exclusive access
}

/// Coordination channels for daemon services
#[derive(Debug)]
pub struct CoordinationChannels {
    pub command_tx: broadcast::Sender<DaemonCommand>,
    pub response_tx: broadcast::Sender<DaemonResponse>,
}

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
    // Initialize daemon environment
    let daemon_context = initialize_daemon(&fastn_home).await?;
    
    // Set up coordination channels
    let coordination = setup_coordination_channels().await?;
    
    // Start P2P networking layer
    start_p2p_service(&daemon_context, &coordination).await?;
    
    // Start control socket service
    start_control_service(fastn_home, &coordination).await?;
    
    // Run main coordination loop
    run_coordination_loop(coordination).await?;
    
    Ok(())
}

/// Initialize daemon environment with singleton lock protection
async fn initialize_daemon(fastn_home: &PathBuf) -> Result<DaemonContext, Box<dyn std::error::Error>> {
    // Ensure FASTN_HOME directory exists
    tokio::fs::create_dir_all(fastn_home).await?;
    
    // Create/open lock file for singleton protection
    let lock_path = fastn_home.join("lock.file");
    let lock_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&lock_path)?;
        
    // Try to acquire exclusive lock - fail immediately if another daemon running
    if let Err(e) = lock_file.try_lock_exclusive() {
        return Err(format!(
            "❌ Another daemon is already running (lock file: {})\n   Error: {}\n   Shutdown the existing daemon first.", 
            lock_path.display(), 
            e
        ).into());
    }
    
    println!("🔒 Acquired exclusive daemon lock: {}", lock_path.display());
    
    // Generate runtime private key (not persistent for MVP)
    let private_key = fastn_id52::SecretKey::generate();
    let peer_id = private_key.public_key();
    
    println!("🔑 Generated runtime daemon key");
    println!("   Peer ID: {}", peer_id.id52());
    println!("   Lock file: {}", lock_path.display());
    
    Ok(DaemonContext {
        private_key,
        peer_id,
        fastn_home: fastn_home.clone(),
        _lock_file: lock_file, // Keep file open to maintain lock
    })
}

/// Set up broadcast channels for coordination between services
async fn setup_coordination_channels() -> Result<CoordinationChannels, Box<dyn std::error::Error>> {
    todo!("Create command and response broadcast channels for control<->P2P coordination");
}

/// Start the P2P networking service
async fn start_p2p_service(
    _daemon_context: &DaemonContext,
    _coordination: &CoordinationChannels,
) -> Result<(), Box<dyn std::error::Error>> {
    todo!("Initialize iroh endpoint, start protocol handlers, spawn P2P service task");
}

/// Start the control socket service
async fn start_control_service(
    _fastn_home: PathBuf,
    _coordination: &CoordinationChannels,
) -> Result<(), Box<dyn std::error::Error>> {
    todo!("Create Unix socket, bind listener, spawn control service task");
}

/// Run the main coordination loop that handles service lifecycle
async fn run_coordination_loop(
    _coordination: CoordinationChannels,
) -> Result<(), Box<dyn std::error::Error>> {
    todo!("Coordinate between control socket and P2P services, handle shutdown signals");
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