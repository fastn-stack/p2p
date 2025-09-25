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
pub mod protocol_trait;

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
    /// Reload identity configurations from disk
    ReloadIdentities,
    /// Set an identity online/offline
    SetIdentityState {
        identity: String,
        online: bool,
    },
    /// Add a protocol binding to an identity
    AddProtocol {
        identity: String,
        protocol: String,
        bind_alias: String,
        config: serde_json::Value,
    },
    /// Remove a protocol binding from an identity
    RemoveProtocol {
        identity: String,
        protocol: String,
        bind_alias: String,
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
    /// Identity configurations reloaded
    IdentitiesReloaded {
        total: usize,
        online: usize,
    },
    /// Identity state changed successfully
    IdentityStateChanged {
        identity: String,
        online: bool,
    },
    /// Protocol binding added successfully
    ProtocolAdded {
        identity: String,
        protocol: String,
        bind_alias: String,
    },
    /// Protocol binding removed successfully
    ProtocolRemoved {
        identity: String,
        protocol: String,
        bind_alias: String,
    },
    /// Operation error
    OperationError {
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

/// Initialize daemon environment with identity management
async fn initialize_daemon(fastn_home: &PathBuf) -> Result<DaemonContext, Box<dyn std::error::Error>> {
    // Use generic server utilities
    fastn_p2p::server::ensure_fastn_home(fastn_home).await?;
    let lock_file = fastn_p2p::server::acquire_singleton_lock(fastn_home).await?;
    
    // Load all available identity configurations  
    let all_identities = fastn_p2p::server::load_all_identities(fastn_home).await?;
    
    if all_identities.is_empty() {
        println!("⚠️  No identities found in {}/identities/", fastn_home.display());
        println!("   Daemon will start and wait for identities to be created");
        println!("   Create an identity with: fastn-p2p create-identity <alias>");
    } else {
        // Show status of all identities
        let online_count = all_identities.iter().filter(|id| id.online).count();
        let total_protocols: usize = all_identities.iter()
            .filter(|id| id.online)
            .map(|id| id.protocols.len())
            .sum();
            
        println!("🔑 Loaded {} identities ({} online)", all_identities.len(), online_count);
        
        for identity in &all_identities {
            let status_icon = if identity.online { "🟢" } else { "🔴" };
            let status_text = if identity.online { "ONLINE" } else { "OFFLINE" };
            
            println!("   {} {} ({}) - {} protocols", 
                    status_icon, 
                    identity.alias, 
                    status_text,
                    identity.protocols.len());
        }
        
        if online_count == 0 {
            println!("⚠️  No online identities - no P2P services will be started");
            println!("   Enable identities with: fastn-p2p identity-online <alias>");
        } else {
            println!("✅ Will start {} P2P services for online identities", total_protocols);
        }
    }
    
    Ok(DaemonContext {
        fastn_home: fastn_home.clone(),
        _lock_file: lock_file,
    })
}

/// Set up broadcast channels for coordination between services
async fn setup_coordination_channels() -> Result<CoordinationChannels, Box<dyn std::error::Error>> {
    // Create broadcast channels for communication between control socket and P2P services
    let (command_tx, _command_rx) = broadcast::channel::<DaemonCommand>(100);
    let (response_tx, _response_rx) = broadcast::channel::<DaemonResponse>(100);
    
    println!("📡 Created coordination channels");
    println!("   Command channel: 100 message buffer");
    println!("   Response channel: 100 message buffer");
    
    Ok(CoordinationChannels {
        command_tx,
        response_tx,
    })
}

/// Start the P2P networking service
async fn start_p2p_service(
    daemon_context: &DaemonContext,
    coordination: &CoordinationChannels,
) -> Result<(), Box<dyn std::error::Error>> {
    // Load current online identities for P2P services
    let online_identities: Vec<_> = fastn_p2p::server::load_all_identities(&daemon_context.fastn_home)
        .await?
        .into_iter()
        .filter(|identity| identity.online)
        .collect();
    
    if online_identities.is_empty() {
        println!("📡 P2P service: No online identities - waiting for activation");
        // Still spawn the P2P task to handle future commands
    } else {
        let total_protocols: usize = online_identities.iter().map(|id| id.protocols.len()).sum();
        println!("📡 P2P service: Starting {} protocols for {} online identities", 
                total_protocols, online_identities.len());
        
        for identity in &online_identities {
            println!("   🟢 {} - {} protocols", identity.alias, identity.protocols.len());
        }
    }
    
    // Spawn P2P service task
    let command_rx = coordination.command_tx.subscribe();
    let response_tx = coordination.response_tx.clone();
    let fastn_home = daemon_context.fastn_home.clone();
    
    tokio::spawn(async move {
        if let Err(e) = p2p::run(fastn_home, command_rx, response_tx).await {
            eprintln!("❌ P2P service error: {}", e);
        }
    });
    
    println!("✅ P2P service task spawned");
    Ok(())
}

/// Start the control socket service
async fn start_control_service(
    fastn_home: PathBuf,
    coordination: &CoordinationChannels,
) -> Result<(), Box<dyn std::error::Error>> {
    // Spawn control socket server task
    let command_tx = coordination.command_tx.clone();
    let response_rx = coordination.response_tx.subscribe();
    
    tokio::spawn(async move {
        if let Err(e) = control::run(fastn_home, command_tx, response_rx).await {
            eprintln!("❌ Control socket service error: {}", e);
        }
    });
    
    println!("✅ Control socket service task spawned");
    Ok(())
}

/// Run the main coordination loop that handles service lifecycle
async fn run_coordination_loop(
    _coordination: CoordinationChannels,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Starting main coordination loop");
    println!("   - P2P service: Running in background");
    println!("   - Control socket: Running in background");
    println!("   - Coordination: Active via broadcast channels");
    
    // Keep the daemon running - both services are now spawned
    // TODO: Handle shutdown signals, coordinate service lifecycle
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        // Services are running in background tasks
        // Main loop keeps daemon alive and can handle coordination
    }
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