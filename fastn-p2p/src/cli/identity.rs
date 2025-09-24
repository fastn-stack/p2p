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
    
    // Generate new identity
    let secret_key = fastn_id52::SecretKey::generate();
    let public_key = secret_key.public_key();
    
    println!("🔑 Generated new identity: {}", alias);
    println!("   Peer ID: {}", public_key.id52());
    
    // Save to identities directory using alias name
    let identity_path = identities_dir.join(format!("{}.key", alias));
    
    if identity_path.exists() {
        return Err(format!("Identity '{}' already exists at: {}", alias, identity_path.display()).into());
    }
    
    // Use save_to_dir method for proper storage
    secret_key.save_to_dir(&identities_dir, &alias)?;
    
    println!("💾 Saved identity to: {}", identity_path.display());
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
    
    // Parse JSON config
    let config: serde_json::Value = serde_json::from_str(&config_json)
        .map_err(|e| format!("Invalid JSON config: {}", e))?;
    
    // Load existing identity config
    let mut identity_config = fastn_p2p::server::IdentityConfig::load_from_dir(&identities_dir, &identity).await
        .map_err(|e| format!("Identity '{}' not found: {}", identity, e))?;
    
    // Check if binding already exists
    if identity_config.protocols.iter().any(|p| p.protocol == protocol && p.bind_alias == bind_alias) {
        return Err(format!("Protocol binding '{}' as '{}' already exists for identity '{}'", protocol, bind_alias, identity).into());
    }
    
    // Add protocol binding
    identity_config = identity_config.add_protocol(protocol.clone(), bind_alias.clone(), config.clone());
    
    // Save updated config
    identity_config.save_to_dir(&identities_dir).await?;
    
    println!("➕ Added protocol binding to identity '{}'", identity);
    println!("   Protocol: {} as '{}'", protocol, bind_alias);
    println!("   Config: {}", serde_json::to_string_pretty(&config)?);
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

/// List all identities and their protocol configurations
pub async fn list_identities(
    fastn_home: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let identity_configs = fastn_p2p::server::load_all_identities(&fastn_home).await?;
    
    if identity_configs.is_empty() {
        println!("📭 No identities found in {}/identities/", fastn_home.display());
        println!("   Create an identity with: fastn-p2p create-identity <alias>");
        return Ok(());
    }
    
    println!("📋 Found {} identities in {}/identities/:", identity_configs.len(), fastn_home.display());
    println!();
    
    for identity in &identity_configs {
        println!("🔑 Identity: {}", identity.alias);
        println!("   Peer ID: {}", identity.secret_key.public_key().id52());
        println!("   Protocols: {}", identity.protocols.len());
        
        if identity.protocols.is_empty() {
            println!("     (no protocols configured)");
        } else {
            for protocol in &identity.protocols {
                println!("     - {} as '{}' (config: {} bytes)", 
                        protocol.protocol, 
                        protocol.bind_alias,
                        protocol.config.to_string().len());
            }
        }
        println!();
    }
    
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