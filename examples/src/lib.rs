//! Shared utilities for P2P examples

/// Parse a key from string or generate a new one
pub fn key_from_str_or_generate(key_str: Option<&str>) -> Result<fastn_id52::SecretKey, Box<dyn std::error::Error>> {
    match key_str {
        Some(s) => Ok(s.parse()?),
        None => Ok(fastn_id52::SecretKey::generate()),
    }
}

/// Parse a peer ID from string
pub fn parse_peer_id(id52_str: &str) -> Result<fastn_id52::PublicKey, Box<dyn std::error::Error>> {
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

/// Parsed arguments with private key already generated
pub struct ParsedArgs {
    pub mode: ParsedMode,
}

/// Mode with private key already generated for server
pub enum ParsedMode {
    Server {
        private_key: fastn_id52::SecretKey,
        config: Vec<String>,
    },
    Client {
        target: fastn_id52::PublicKey,
        config: Vec<String>,
    },
}

/// Parse CLI arguments and handle key generation/parsing automatically
pub fn parse_cli() -> Result<ParsedArgs, Box<dyn std::error::Error>> {
    let args = <Args as clap::Parser>::parse();
    
    let mode = match args.mode {
        Mode::Server { key, config } => {
            let private_key = key_from_str_or_generate(key.as_deref())?;
            ParsedMode::Server { private_key, config }
        }
        Mode::Client { target, config } => {
            let target_key = parse_peer_id(&target)?;
            ParsedMode::Client { target: target_key, config }
        }
    };
    
    Ok(ParsedArgs { mode })
}

// Clean re-exports for examples
pub use ParsedMode::Server;
pub use ParsedMode::Client;