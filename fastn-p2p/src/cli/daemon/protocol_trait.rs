//! Protocol trait for standardizing protocol lifecycle management
//!
//! This trait defines the standard interface that all protocols must implement
//! for proper integration with the fastn-p2p daemon.

use std::path::PathBuf;

/// Protocol lifecycle management trait
/// 
/// All protocols must implement this trait to integrate with the daemon.
/// The trait provides a standardized lifecycle for protocol configuration
/// and service management.
#[async_trait::async_trait]
pub trait Protocol {
    /// Protocol name (e.g., "Mail", "Chat", "FileShare")
    const NAME: &'static str;
    
    /// Initialize protocol configuration for first-time setup
    /// 
    /// Creates the protocol's config directory structure and writes default
    /// configuration files. This is called when a protocol is first added
    /// to an identity.
    /// 
    /// # Parameters
    /// * `bind_alias` - The alias for this protocol instance (e.g., "default", "backup")
    /// * `config_path` - Directory path where config files should be created
    /// 
    /// # Directory Structure Created
    /// ```
    /// config_path/
    /// ├── config.json           # Main protocol configuration
    /// ├── data/                 # Protocol-specific data directory (optional)
    /// └── logs/                 # Protocol-specific logs (optional)
    /// ```
    async fn init(
        bind_alias: &str,
        config_path: &PathBuf,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    
    /// Load protocol and start P2P services
    /// 
    /// Reads configuration from config_path and starts the protocol's P2P
    /// listeners and handlers. This is called when the daemon starts or
    /// when an identity comes online.
    /// 
    /// # Parameters  
    /// * `bind_alias` - The alias for this protocol instance
    /// * `config_path` - Directory path containing config files
    /// * `identity_key` - The identity's secret key for P2P operations
    async fn load(
        bind_alias: &str,
        config_path: &PathBuf,
        identity_key: &fastn_id52::SecretKey,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    
    /// Reload protocol configuration and restart services
    /// 
    /// Re-reads configuration files and restarts P2P services with updated
    /// settings. This allows configuration changes without full daemon restart.
    /// 
    /// # Parameters
    /// * `bind_alias` - The alias for this protocol instance  
    /// * `config_path` - Directory path containing updated config files
    async fn reload(
        bind_alias: &str,
        config_path: &PathBuf,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    
    /// Stop protocol services cleanly
    /// 
    /// Performs clean shutdown of all P2P listeners and handlers for this
    /// protocol instance. This is called when an identity goes offline or
    /// when a protocol is removed.
    /// 
    /// # Parameters
    /// * `bind_alias` - The alias for this protocol instance
    async fn stop(
        bind_alias: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    
    /// Check protocol configuration without affecting runtime
    /// 
    /// Validates configuration files and reports any issues without changing
    /// running services. This is useful for configuration validation and
    /// troubleshooting.
    /// 
    /// # Parameters
    /// * `bind_alias` - The alias for this protocol instance
    /// * `config_path` - Directory path containing config files to validate
    async fn check(
        bind_alias: &str,
        config_path: &PathBuf,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

/// Registry of available protocols
/// 
/// This function returns a list of all protocols that can be loaded by the daemon.
/// Production applications would register their own protocols here.
pub fn get_available_protocols() -> Vec<&'static str> {
    vec![
        super::protocols::echo::EchoProtocol::NAME,
        super::protocols::shell::ShellProtocol::NAME,
    ]
}

/// Load a protocol by name using the trait interface
/// 
/// This function dispatches to the appropriate protocol implementation
/// based on the protocol name.
pub async fn load_protocol(
    protocol_name: &str,
    bind_alias: &str,
    config_path: &PathBuf,
    identity_key: &fastn_id52::SecretKey,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match protocol_name {
        "Echo" => {
            super::protocols::echo::EchoProtocol::load(bind_alias, config_path, identity_key).await
        }
        "Shell" => {
            super::protocols::shell::ShellProtocol::load(bind_alias, config_path, identity_key).await
        }
        _ => {
            Err(format!("Unknown protocol: {}", protocol_name).into())
        }
    }
}

/// Initialize a protocol by name using the trait interface
pub async fn init_protocol(
    protocol_name: &str,
    bind_alias: &str,
    config_path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match protocol_name {
        "Echo" => {
            super::protocols::echo::EchoProtocol::init(bind_alias, config_path).await
        }
        "Shell" => {
            super::protocols::shell::ShellProtocol::init(bind_alias, config_path).await
        }
        _ => {
            Err(format!("Unknown protocol: {}", protocol_name).into())
        }
    }
}

/// Check a protocol by name using the trait interface
pub async fn check_protocol(
    protocol_name: &str,
    bind_alias: &str,
    config_path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match protocol_name {
        "Echo" => {
            super::protocols::echo::EchoProtocol::check(bind_alias, config_path).await
        }
        "Shell" => {
            super::protocols::shell::ShellProtocol::check(bind_alias, config_path).await
        }
        _ => {
            Err(format!("Unknown protocol: {}", protocol_name).into())
        }
    }
}