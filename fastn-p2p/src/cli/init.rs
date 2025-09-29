//! FASTN_HOME initialization

use std::path::PathBuf;

/// Initialize FASTN_HOME directory structure
pub async fn run(fastn_home: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    // Create basic directory structure
    tokio::fs::create_dir_all(&fastn_home).await?;
    tokio::fs::create_dir_all(fastn_home.join("identities")).await?;
    
    println!("✅ FASTN_HOME initialized: {}", fastn_home.display());
    println!("📁 Directory structure created");
    println!("");
    println!("Next steps:");
    println!("  1. Create identity: fastn-p2p create-identity alice");
    println!("  2. Add protocol: fastn-p2p add-protocol alice --protocol echo.fastn.com --config '{{\"max_length\": 1000}}'");
    println!("  3. Set online: fastn-p2p identity-online alice");
    println!("  4. Start protocol server: cargo run --bin <protocol_server>");
    
    Ok(())
}