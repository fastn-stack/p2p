//! Generic daemon functionality for fastn-p2p servers
//!
//! This module provides utilities that any fastn-p2p server application will need:
//! - FASTN_HOME directory management
//! - Identity loading and management
//! - Generic multi-identity, multi-protocol server setup

use std::path::PathBuf;

/// Protocol binding configuration with file-based config
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProtocolBinding {
    pub protocol: String,
    pub bind_alias: String,
    pub config_path: PathBuf,
}

/// Identity with protocol bindings and online/offline state
#[derive(Debug, Clone)]
pub struct IdentityConfig {
    pub alias: String,
    pub secret_key: fastn_id52::SecretKey,
    pub protocols: Vec<ProtocolBinding>,
    pub online: bool,
}

/// Serializable version of IdentityConfig (without secret key)
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct IdentityConfigSerialized {
    alias: String,
    protocols: Vec<ProtocolBinding>,
    #[serde(default = "default_online_true")]
    online: bool,
}

fn default_online_true() -> bool {
    true
}

impl IdentityConfig {
    /// Create a new identity config with no protocols (online by default)
    pub fn new(alias: String, secret_key: fastn_id52::SecretKey) -> Self {
        Self {
            alias,
            secret_key,
            protocols: Vec::new(),
            online: true,
        }
    }
    
    /// Add a protocol binding to this identity
    pub fn add_protocol(mut self, protocol: String, bind_alias: String, config_path: PathBuf) -> Self {
        self.protocols.push(ProtocolBinding {
            protocol,
            bind_alias,
            config_path,
        });
        self
    }
    
    /// Save this identity config to conventional directory structure
    pub async fn save_to_dir(&self, identities_dir: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let identity_dir = identities_dir.join(&self.alias);
        tokio::fs::create_dir_all(&identity_dir).await?;
        
        // Save secret key inside identity directory if it doesn't exist yet
        let key_path = identity_dir.join("identity.private-key");
        if !key_path.exists() {
            self.secret_key.save_to_dir(&identity_dir, "identity")?;
        }
        
        // Save online/offline state as marker file
        let online_marker = identity_dir.join("online");
        if self.online {
            tokio::fs::write(&online_marker, "").await?;
        } else if online_marker.exists() {
            tokio::fs::remove_file(&online_marker).await?;
        }
        
        Ok(())
    }
    
    /// Load identity config from conventional directory structure
    pub async fn load_from_conventional_dir(identity_dir: &PathBuf, alias: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // Load the secret key from inside identity directory
        let (_id52, secret_key) = fastn_id52::SecretKey::load_from_dir(identity_dir, "identity")?;
        
        // Check if identity is online (online file exists)
        let online_marker = identity_dir.join("online");
        let online = online_marker.exists();
        
        // Discover all protocol configurations by scanning protocols/ directory
        let mut protocols = Vec::new();
        let protocols_dir = identity_dir.join("protocols");
        
        if protocols_dir.exists() {
            protocols = discover_protocol_bindings(&protocols_dir).await?;
        }
        
        println!("🔍 Discovered identity '{}': {} protocols, {}", 
                alias, 
                protocols.len(),
                if online { "ONLINE" } else { "OFFLINE" });
        
        Ok(IdentityConfig {
            alias: alias.to_string(),
            secret_key,
            protocols,
            online,
        })
    }
    
    /// Legacy load method for backward compatibility
    pub async fn load_from_dir(identities_dir: &PathBuf, alias: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // Try new conventional structure first
        let identity_dir = identities_dir.join(alias);
        if identity_dir.exists() {
            return Self::load_from_conventional_dir(&identity_dir, alias).await;
        }
        
        // Fall back to old structure
        let (_id52, secret_key) = fastn_id52::SecretKey::load_from_dir(identities_dir, alias)?;
        let config_path = identities_dir.join(format!("{}.config.json", alias));
        let mut config = if config_path.exists() {
            let config_json = tokio::fs::read_to_string(&config_path).await?;
            let serialized: IdentityConfigSerialized = serde_json::from_str(&config_json)?;
            IdentityConfig {
                alias: serialized.alias,
                secret_key,
                protocols: serialized.protocols,
                online: serialized.online,
            }
        } else {
            IdentityConfig::new(alias.to_string(), secret_key)
        };
        
        config.alias = alias.to_string();
        Ok(config)
    }
}

/// Server configuration for multiple identities and protocols
pub type ServerConfig = Vec<IdentityConfig>;

/// Get or create FASTN_HOME directory
pub async fn ensure_fastn_home(fastn_home: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    tokio::fs::create_dir_all(fastn_home).await?;
    tokio::fs::create_dir_all(fastn_home.join("identities")).await?;
    Ok(())
}

/// Load all identity configurations using conventional directory structure
pub async fn load_all_identities(
    fastn_home: &PathBuf,
) -> Result<Vec<IdentityConfig>, Box<dyn std::error::Error>> {
    let identities_dir = fastn_home.join("identities");
    
    if !identities_dir.exists() {
        return Ok(vec![]);
    }
    
    let mut identities = Vec::new();
    let mut dir_entries = tokio::fs::read_dir(&identities_dir).await?;
    
    while let Some(entry) = dir_entries.next_entry().await? {
        let identity_dir = entry.path();
        
        if identity_dir.is_dir() {
            if let Some(alias) = identity_dir.file_name().and_then(|n| n.to_str()) {
                match IdentityConfig::load_from_conventional_dir(&identity_dir, alias).await {
                    Ok(identity_config) => {
                        identities.push(identity_config);
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

/// Discover protocol bindings from conventional protocols/ directory structure
async fn discover_protocol_bindings(protocols_dir: &PathBuf) -> Result<Vec<ProtocolBinding>, Box<dyn std::error::Error>> {
    let mut bindings = Vec::new();
    let mut protocol_entries = tokio::fs::read_dir(protocols_dir).await?;
    
    while let Some(protocol_entry) = protocol_entries.next_entry().await? {
        let protocol_dir = protocol_entry.path();
        
        if protocol_dir.is_dir() {
            if let Some(protocol_name) = protocol_dir.file_name().and_then(|n| n.to_str()) {
                // Scan for bind aliases within this protocol directory
                let mut alias_entries = tokio::fs::read_dir(&protocol_dir).await?;
                
                while let Some(alias_entry) = alias_entries.next_entry().await? {
                    let alias_dir = alias_entry.path();
                    
                    if alias_dir.is_dir() {
                        if let Some(bind_alias) = alias_dir.file_name().and_then(|n| n.to_str()) {
                            // Check if config.json exists
                            let config_file = alias_dir.join("config.json");
                            if config_file.exists() {
                                bindings.push(ProtocolBinding {
                                    protocol: protocol_name.to_string(),
                                    bind_alias: bind_alias.to_string(),
                                    config_path: alias_dir.clone(),
                                });
                                
                                println!("    📡 Found: {} as '{}' ({})", 
                                        protocol_name, 
                                        bind_alias, 
                                        alias_dir.display());
                            }
                        }
                    }
                }
            }
        }
    }
    
    Ok(bindings)
}