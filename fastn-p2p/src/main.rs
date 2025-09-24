//! fastn-p2p: P2P daemon and client
//!
//! This binary provides both daemon and client functionality for P2P communication.
//! It uses Unix domain sockets for communication between client and daemon.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod cli;

#[derive(Parser)]
#[command(name = "fastn-p2p")]
#[command(about = "P2P daemon and client for fastn")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the P2P daemon in foreground mode
    Daemon {
        /// Custom FASTN_HOME directory (defaults to FASTN_HOME env var or ~/.fastn)
        #[arg(long, env = "FASTN_HOME")]
        home: Option<PathBuf>,
    },
    /// Make a request/response call to a peer
    Call {
        /// Target peer ID52
        peer: String,
        /// Protocol name
        protocol: String,
        /// Custom FASTN_HOME directory (defaults to FASTN_HOME env var or ~/.fastn)
        #[arg(long, env = "FASTN_HOME")]
        home: Option<PathBuf>,
    },
    /// Open a bidirectional stream to a peer
    Stream {
        /// Target peer ID52
        peer: String,
        /// Protocol name
        protocol: String,
        /// Custom FASTN_HOME directory (defaults to FASTN_HOME env var or ~/.fastn)
        #[arg(long, env = "FASTN_HOME")]
        home: Option<PathBuf>,
    },
    /// Create a new identity and save it to FASTN_HOME/identities/
    CreateIdentity {
        /// Identity alias name
        alias: String,
        /// Custom FASTN_HOME directory (defaults to FASTN_HOME env var or ~/.fastn)
        #[arg(long, env = "FASTN_HOME")]
        home: Option<PathBuf>,
    },
}

#[fastn_p2p::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Daemon { home } => {
            let fastn_home = cli::get_fastn_home(home)?;
            println!("🚀 Starting fastn-p2p daemon");
            println!("📁 FASTN_HOME: {}", fastn_home.display());
            cli::daemon::run(fastn_home).await
        }
        Commands::Call { peer, protocol, home } => {
            let fastn_home = cli::get_fastn_home(home)?;
            cli::client::call(fastn_home, peer, protocol).await
        }
        Commands::Stream { peer, protocol, home } => {
            let fastn_home = cli::get_fastn_home(home)?;
            cli::client::stream(fastn_home, peer, protocol).await
        }
        Commands::CreateIdentity { alias, home } => {
            let fastn_home = cli::get_fastn_home(home)?;
            cli::identity::create_identity(fastn_home, alias).await
        }
    }
}