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