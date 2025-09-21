//! Shared utilities for P2P examples

use std::path::Path;

/// Parse a key from string or generate a new one
pub fn key_from_str_or_generate(
    key_str: Option<&str>,
) -> Result<fastn_p2p::SecretKey, Box<dyn std::error::Error>> {
    match key_str {
        Some(s) => Ok(s.parse()?),
        None => Ok(fastn_p2p::SecretKey::generate()),
    }
}

/// Get or create a persistent private key for the given example
/// Stores keys in .fastn-{example_name}.key files for consistent server IDs
pub fn get_or_create_persistent_key(example_name: &str) -> fastn_p2p::SecretKey {
    let key_file = format!(".fastn-{}.key", example_name);
    
    // Try to load existing key
    if Path::new(&key_file).exists() {
        if let Ok(key_data) = std::fs::read_to_string(&key_file) {
            if let Ok(key) = key_data.trim().parse::<fastn_p2p::SecretKey>() {
                println!("🔑 Using persistent key from: {}", key_file);
                return key;
            }
        }
        println!("⚠️  Could not load key from {}, generating new one", key_file);
    }
    
    // Generate new key
    let key = fastn_p2p::SecretKey::generate();
    
    // Try to save it
    match std::fs::write(&key_file, key.to_string()) {
        Ok(_) => {
            println!("🔑 Generated and saved persistent key to: {}", key_file);
            println!("   (Server ID will remain consistent across restarts)");
        }
        Err(e) => {
            println!("⚠️  Could not save key to {} ({}), server ID will change on restart", key_file, e);
        }
    }
    
    key
}

/// Parse a peer ID from string
pub fn parse_peer_id(id52_str: &str) -> Result<fastn_p2p::PublicKey, Box<dyn std::error::Error>> {
    Ok(id52_str.parse()?)
}

/// Empty args for when no additional config is needed
#[derive(clap::Args)]
pub struct NoArgs {}

/// Simple CLI arguments for P2P examples
#[derive(clap::Parser)]
pub struct Args {
    #[command(subcommand)]
    pub mode: Mode,
}

/// Simple mode enum for client/server patterns
#[derive(clap::Subcommand)]
pub enum Mode {
    /// Start server mode
    Server {
        /// Optional private key
        #[arg(long)]
        key: Option<String>,
        /// Additional server arguments
        #[arg(trailing_var_arg = true)]
        config: Vec<String>,
    },
    /// Start client mode  
    Client {
        /// Target server ID52
        target: String,
        /// Additional client arguments
        #[arg(trailing_var_arg = true)]
        config: Vec<String>,
    },
}

/// Parsed mode with keys already generated/parsed
pub enum ParsedMode {
    Server {
        private_key: fastn_p2p::SecretKey,
        config: Vec<String>,
    },
    Client {
        target: fastn_p2p::PublicKey,
        config: Vec<String>,
    },
}

/// Parse CLI arguments and handle key generation/parsing automatically
pub fn parse_cli() -> Result<ParsedMode, Box<dyn std::error::Error>> {
    let args = <Args as clap::Parser>::parse();

    match args.mode {
        Mode::Server { key, config } => {
            let private_key = key_from_str_or_generate(key.as_deref())?;
            Ok(ParsedMode::Server {
                private_key,
                config,
            })
        }
        Mode::Client { target, config } => {
            let target_key = parse_peer_id(&target)?;
            Ok(ParsedMode::Client {
                target: target_key,
                config,
            })
        }
    }
}

// Clean re-exports for examples
pub use ParsedMode::Client;
pub use ParsedMode::Server;
