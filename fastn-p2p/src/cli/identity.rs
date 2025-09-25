//! Identity management for fastn-p2p
//!
//! Handles creation, storage, and loading of persistent identities.

use std::path::PathBuf;

/// Create a new identity and save it with the given alias
pub async fn create_identity(
    fastn_home: PathBuf,
    alias: String,
) -> Result<(), Box<dyn std::error::Error>> {
    // Ensure identities directory exists
    let identities_dir = fastn_home.join("identities");
    tokio::fs::create_dir_all(&identities_dir).await?;
    
    // Create identity-specific directory
    let identity_dir = identities_dir.join(&alias);
    if identity_dir.exists() {
        return Err(format!("Identity '{}' already exists at: {}", alias, identity_dir.display()).into());
    }
    
    tokio::fs::create_dir_all(&identity_dir).await?;
    
    // Generate new identity
    let secret_key = fastn_id52::SecretKey::generate();
    let public_key = secret_key.public_key();
    
    println!("🔑 Generated new identity: {}", alias);
    println!("   Peer ID: {}", public_key.id52());
    
    // Save secret key inside identity directory with standard name "identity"
    secret_key.save_to_dir(&identity_dir, "identity")?;
    
    println!("💾 Saved identity to: {}", identity_dir.display());
    println!("✅ Identity '{}' created successfully", alias);
    
    Ok(())
}

/// Add a protocol binding to an identity
pub async fn add_protocol(
    fastn_home: PathBuf,
    identity: String,
    protocol: String,
    bind_alias: String,
    config_json: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let identities_dir = fastn_home.join("identities");
    
    // Parse JSON config for initial setup
    let config: serde_json::Value = serde_json::from_str(&config_json)
        .map_err(|e| format!("Invalid JSON config: {}", e))?;
    
    // Create protocol config directory
    let protocol_config_path = identities_dir.join(&identity).join("protocols").join(&bind_alias);
    tokio::fs::create_dir_all(&protocol_config_path).await?;
    
    // Load existing identity config
    let identity_config = fastn_p2p::server::IdentityConfig::load_from_dir(&identities_dir, &identity).await
        .map_err(|e| format!("Identity '{}' not found: {}", identity, e))?;
    
    // Check if binding already exists
    if identity_config.protocols.iter().any(|p| p.protocol == protocol && p.bind_alias == bind_alias) {
        return Err(format!("Protocol binding '{}' as '{}' already exists for identity '{}'", protocol, bind_alias, identity).into());
    }
    
    // Initialize the protocol handler - just create the directory and config for now
    // TODO: Hook into serve_all protocol handlers for proper initialization
    tokio::fs::create_dir_all(&protocol_config_path).await?;
    
    // Write the initial config JSON to the protocol directory
    let config_file = protocol_config_path.join(format!("{}.json", protocol.to_lowercase()));
    tokio::fs::write(&config_file, serde_json::to_string_pretty(&config)?).await?;
    
    // Protocol configuration is already saved to the config file above
    // The identity directory structure auto-discovers protocols via directory scanning
    
    println!("➕ Added protocol binding to identity '{}'", identity);
    println!("   Protocol: {} as '{}'", protocol, bind_alias);
    println!("   Config path: {}", protocol_config_path.display());
    println!("   Config file: {}", config_file.display());
    println!("✅ Protocol binding saved");
    
    Ok(())
}

/// Remove a protocol binding from an identity
pub async fn remove_protocol(
    fastn_home: PathBuf,
    identity: String,
    protocol: String,
    bind_alias: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let identities_dir = fastn_home.join("identities");
    
    // Load existing identity config
    let mut identity_config = fastn_p2p::server::IdentityConfig::load_from_dir(&identities_dir, &identity).await
        .map_err(|e| format!("Identity '{}' not found: {}", identity, e))?;
    
    // Find and remove the protocol binding
    let original_count = identity_config.protocols.len();
    identity_config.protocols.retain(|p| !(p.protocol == protocol && p.bind_alias == bind_alias));
    
    if identity_config.protocols.len() == original_count {
        return Err(format!("Protocol binding '{}' as '{}' not found for identity '{}'", protocol, bind_alias, identity).into());
    }
    
    // Save updated config
    identity_config.save_to_dir(&identities_dir).await?;
    
    println!("➖ Removed protocol binding from identity '{}'", identity);
    println!("   Protocol: {} as '{}'", protocol, bind_alias);
    println!("✅ Protocol binding removed");
    
    Ok(())
}

/// Set an identity online (enable its protocols)
pub async fn set_identity_online(
    fastn_home: PathBuf,
    identity: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let identities_dir = fastn_home.join("identities");
    
    // Load identity config
    let mut identity_config = fastn_p2p::server::IdentityConfig::load_from_dir(&identities_dir, &identity).await
        .map_err(|e| format!("Identity '{}' not found: {}", identity, e))?;
    
    if identity_config.online {
        println!("ℹ️  Identity '{}' is already online", identity);
        return Ok(());
    }
    
    // Set online and save
    identity_config.online = true;
    identity_config.save_to_dir(&identities_dir).await?;
    
    println!("🟢 Identity '{}' is now ONLINE", identity);
    println!("   {} protocols will be enabled when daemon starts", identity_config.protocols.len());
    
    Ok(())
}

/// Set an identity offline (disable its protocols)
pub async fn set_identity_offline(
    fastn_home: PathBuf,
    identity: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let identities_dir = fastn_home.join("identities");
    
    // Load identity config
    let mut identity_config = fastn_p2p::server::IdentityConfig::load_from_dir(&identities_dir, &identity).await
        .map_err(|e| format!("Identity '{}' not found: {}", identity, e))?;
    
    if !identity_config.online {
        println!("ℹ️  Identity '{}' is already offline", identity);
        return Ok(());
    }
    
    // Set offline and save
    identity_config.online = false;
    identity_config.save_to_dir(&identities_dir).await?;
    
    println!("🔴 Identity '{}' is now OFFLINE", identity);
    println!("   {} protocols will be disabled", identity_config.protocols.len());
    println!("   Restart daemon for changes to take effect");
    
    Ok(())
}

/// Load all identities from FASTN_HOME/identities/ directory
pub async fn load_all_identities(
    fastn_home: &PathBuf,
) -> Result<Vec<(String, fastn_id52::SecretKey)>, Box<dyn std::error::Error>> {
    let identities_dir = fastn_home.join("identities");
    
    if !identities_dir.exists() {
        println!("📁 No identities directory found: {}", identities_dir.display());
        return Ok(vec![]);
    }
    
    let mut identities = Vec::new();
    let mut dir_entries = tokio::fs::read_dir(&identities_dir).await?;
    
    while let Some(entry) = dir_entries.next_entry().await? {
        let path = entry.path();
        
        if path.extension().and_then(|s| s.to_str()) == Some("private-key") {
            if let Some(file_stem) = path.file_stem().and_then(|s| s.to_str()) {
                match fastn_id52::SecretKey::load_from_dir(&identities_dir, file_stem) {
                    Ok((_id52, secret_key)) => {
                        println!("🔑 Loaded identity '{}': {}", file_stem, secret_key.public_key().id52());
                        identities.push((file_stem.to_string(), secret_key));
                    }
                    Err(e) => {
                        eprintln!("⚠️  Failed to load identity '{}': {}", file_stem, e);
                    }
                }
            }
        }
    }
    
    println!("📋 Loaded {} identities from {}", identities.len(), identities_dir.display());
    Ok(identities)
}