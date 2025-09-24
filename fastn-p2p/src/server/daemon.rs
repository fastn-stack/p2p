//! Generic daemon functionality for fastn-p2p servers
//!
//! This module provides utilities that any fastn-p2p server application will need:
//! - FASTN_HOME directory management
//! - Identity loading and management
//! - Generic multi-identity, multi-protocol server setup

use std::path::PathBuf;

/// Protocol binding configuration
#[derive(Debug, Clone)]
pub struct ProtocolBinding {
    pub protocol: String,
    pub bind_alias: String,
}

/// Identity with protocol bindings
#[derive(Debug, Clone)]
pub struct IdentityConfig {
    pub alias: String,
    pub secret_key: fastn_id52::SecretKey,
    pub protocols: Vec<ProtocolBinding>,
}

/// Server configuration for multiple identities and protocols
pub type ServerConfig = Vec<IdentityConfig>;

/// Get or create FASTN_HOME directory
pub async fn ensure_fastn_home(fastn_home: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    tokio::fs::create_dir_all(fastn_home).await?;
    tokio::fs::create_dir_all(fastn_home.join("identities")).await?;
    Ok(())
}

/// Load all identities from FASTN_HOME/identities/ directory
pub async fn load_all_identities(
    fastn_home: &PathBuf,
) -> Result<Vec<(String, fastn_id52::SecretKey)>, Box<dyn std::error::Error>> {
    let identities_dir = fastn_home.join("identities");
    
    if !identities_dir.exists() {
        return Ok(vec![]);
    }
    
    let mut identities = Vec::new();
    let mut dir_entries = tokio::fs::read_dir(&identities_dir).await?;
    
    while let Some(entry) = dir_entries.next_entry().await? {
        let path = entry.path();
        
        if path.extension().and_then(|s| s.to_str()) == Some("private-key") {
            if let Some(alias) = path.file_stem().and_then(|s| s.to_str()) {
                match fastn_id52::SecretKey::load_from_dir(&identities_dir, alias) {
                    Ok((_id52, secret_key)) => {
                        identities.push((alias.to_string(), secret_key));
                    }
                    Err(e) => {
                        eprintln!("⚠️  Failed to load identity '{}': {}", alias, e);
                    }
                }
            }
        }
    }
    
    Ok(identities)
}

/// Generic server function that can be used by any fastn-p2p application
/// 
/// This function sets up a multi-identity, multi-protocol P2P server.
/// Each identity can expose multiple protocols, and each protocol can be
/// bound multiple times with different aliases.
pub async fn run_generic_server(
    fastn_home: PathBuf,
    server_config: ServerConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    // Ensure FASTN_HOME setup
    ensure_fastn_home(&fastn_home).await?;
    
    // Acquire singleton lock
    let _lock_file = acquire_singleton_lock(&fastn_home).await?;
    
    println!("🚀 Starting generic P2P server");
    println!("📁 FASTN_HOME: {}", fastn_home.display());
    println!("🔑 Identities: {}", server_config.len());
    
    for identity in &server_config {
        println!("   Identity '{}': {} (protocols: {})", 
                identity.alias,
                identity.secret_key.public_key().id52(),
                identity.protocols.len());
        
        for protocol in &identity.protocols {
            println!("     - {} as '{}'", protocol.protocol, protocol.bind_alias);
        }
    }
    
    todo!("Initialize P2P listeners for each identity and protocol binding");
}

/// Acquire singleton lock for daemon (shared utility)
pub async fn acquire_singleton_lock(
    fastn_home: &PathBuf,
) -> Result<std::fs::File, Box<dyn std::error::Error>> {
    use fs2::FileExt;
    use std::fs::OpenOptions;
    
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
    Ok(lock_file)
}