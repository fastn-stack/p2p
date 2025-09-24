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
    /// Add a protocol binding to an identity
    AddProtocol {
        /// Identity alias name
        identity: String,
        /// Protocol name to add
        #[arg(long)]
        protocol: String,
        /// Bind alias for this protocol instance (defaults to "default")
        #[arg(long, default_value = "default")]
        alias: String,
        /// Protocol configuration as JSON string
        #[arg(long)]
        config: String,
        /// Custom FASTN_HOME directory (defaults to FASTN_HOME env var or ~/.fastn)
        #[arg(long, env = "FASTN_HOME")]
        home: Option<PathBuf>,
    },
    /// Remove a protocol binding from an identity
    RemoveProtocol {
        /// Identity alias name
        identity: String,
        /// Protocol name to remove
        #[arg(long)]
        protocol: String,
        /// Bind alias to remove (defaults to "default")
        #[arg(long, default_value = "default")]
        alias: String,
        /// Custom FASTN_HOME directory (defaults to FASTN_HOME env var or ~/.fastn)
        #[arg(long, env = "FASTN_HOME")]
        home: Option<PathBuf>,
    },
    /// Show comprehensive daemon and identity status
    Status {
        /// Custom FASTN_HOME directory (defaults to FASTN_HOME env var or ~/.fastn)
        #[arg(long, env = "FASTN_HOME")]
        home: Option<PathBuf>,
    },
    /// Set an identity online (enable its protocols)
    IdentityOnline {
        /// Identity alias name
        identity: String,
        /// Custom FASTN_HOME directory (defaults to FASTN_HOME env var or ~/.fastn)
        #[arg(long, env = "FASTN_HOME")]
        home: Option<PathBuf>,
    },
    /// Set an identity offline (disable its protocols)
    IdentityOffline {
        /// Identity alias name
        identity: String,
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
        Commands::AddProtocol { identity, protocol, alias, config, home } => {
            let fastn_home = cli::get_fastn_home(home)?;
            cli::identity::add_protocol(fastn_home, identity, protocol, alias, config).await
        }
        Commands::RemoveProtocol { identity, protocol, alias, home } => {
            let fastn_home = cli::get_fastn_home(home)?;
            cli::identity::remove_protocol(fastn_home, identity, protocol, alias).await
        }
        Commands::Status { home } => {
            let fastn_home = cli::get_fastn_home(home)?;
            cli::status::show_status(fastn_home).await
        }
        Commands::IdentityOnline { identity, home } => {
            let fastn_home = cli::get_fastn_home(home)?;
            cli::identity::set_identity_online(fastn_home, identity).await
        }
        Commands::IdentityOffline { identity, home } => {
            let fastn_home = cli::get_fastn_home(home)?;
            cli::identity::set_identity_offline(fastn_home, identity).await
        }
    }
}