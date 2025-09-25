//! Status command for showing comprehensive daemon and identity information

use std::path::PathBuf;

/// Show comprehensive daemon and identity status
pub async fn show_status(fastn_home: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 fastn-p2p Status");
    println!("📁 FASTN_HOME: {}", fastn_home.display());
    println!();
    
    // Check if daemon is running
    let daemon_status = check_daemon_status(&fastn_home).await;
    println!("🚀 Daemon: {}", daemon_status);
    
    // Show lock file status
    show_lock_status(&fastn_home).await?;
    println!();
    
    // Show all identities and their configurations
    show_identities_status(&fastn_home).await?;
    
    Ok(())
}

/// Check if daemon is currently running
async fn check_daemon_status(fastn_home: &PathBuf) -> String {
    let socket_path = fastn_home.join("control.sock");
    let lock_path = fastn_home.join("lock.file");
    
    if socket_path.exists() && lock_path.exists() {
        // TODO: Try to connect to control socket to verify daemon is responsive
        "🟢 Running (control socket + lock file present)".to_string()
    } else if lock_path.exists() {
        "🟡 Lock file exists but no control socket (starting up or crashed?)".to_string()
    } else {
        "🔴 Not running".to_string()
    }
}

/// Show lock file information
async fn show_lock_status(fastn_home: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let lock_path = fastn_home.join("lock.file");
    
    if lock_path.exists() {
        let metadata = tokio::fs::metadata(&lock_path).await?;
        let modified = metadata.modified()?;
        let duration = std::time::SystemTime::now().duration_since(modified)?;
        
        println!("🔒 Lock file: {} (created {} seconds ago)", 
                lock_path.display(), 
                duration.as_secs());
    } else {
        println!("🔓 No lock file found");
    }
    
    Ok(())
}

/// Show all identities with their online/offline status and protocol configurations
async fn show_identities_status(fastn_home: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let identity_configs = fastn_p2p::server::load_all_identities(fastn_home).await?;
    
    if identity_configs.is_empty() {
        println!("📭 No identities configured");
        println!("   Create an identity with: fastn-p2p create-identity <alias>");
        return Ok(());
    }
    
    println!("🔑 Identities: {}", identity_configs.len());
    
    for identity in &identity_configs {
        let status_icon = if identity.online { "🟢" } else { "🔴" };
        let status_text = if identity.online { "ONLINE" } else { "OFFLINE" };
        
        println!();
        println!("{} {} ({}) - {}", status_icon, identity.alias, status_text, identity.secret_key.public_key().id52());
        
        if identity.protocols.is_empty() {
            println!("     📭 No protocols configured");
        } else {
            println!("     📡 Protocols: {}", identity.protocols.len());
            for protocol in &identity.protocols {
                let protocol_status = if identity.online { "🟢" } else { "⏸️" };
                println!("       {} {} as '{}' (config: {})", 
                        protocol_status,
                        protocol.protocol, 
                        protocol.bind_alias,
                        protocol.config_path.display());
            }
        }
    }
    
    println!();
    println!("💡 Commands:");
    println!("   fastn-p2p daemon                     # Start daemon");
    println!("   fastn-p2p identity-online <name>     # Enable identity");
    println!("   fastn-p2p identity-offline <name>    # Disable identity");
    
    Ok(())
}