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

/// Generic CLI arguments for P2P examples
#[derive(clap::Parser)]
pub struct Args<CLIENT = NoArgs, SERVER = NoArgs> 
where
    CLIENT: clap::Args,
    SERVER: clap::Args,
{
    #[command(subcommand)]
    pub mode: Mode<CLIENT, SERVER>,
}

/// Generic mode enum for client/server patterns
#[derive(clap::Subcommand)]
pub enum Mode<CLIENT = NoArgs, SERVER = NoArgs> 
where
    CLIENT: clap::Args,
    SERVER: clap::Args,
{
    /// Start server mode
    Server {
        /// Optional private key
        #[arg(long)]
        key: Option<String>,
        #[command(flatten)]
        config: SERVER,
    },
    /// Start client mode  
    Client {
        /// Target server ID52
        target: String,
        #[command(flatten)]
        config: CLIENT,
    },
}