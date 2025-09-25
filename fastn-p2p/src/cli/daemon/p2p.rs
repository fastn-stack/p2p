//! P2P networking layer for the daemon
//!
//! This module handles incoming P2P connections using iroh and routes
//! them to appropriate protocol handlers.

use tokio::sync::broadcast;
use std::collections::HashMap;

use super::{DaemonCommand, DaemonResponse};
use super::protocols::{echo, shell};

/// P2P listener that handles incoming connections and protocol routing
pub async fn run(
    _fastn_home: std::path::PathBuf,
    _command_rx: broadcast::Receiver<DaemonCommand>,
    _response_tx: broadcast::Sender<DaemonResponse>,
) -> Result<(), Box<dyn std::error::Error>> {
    todo!("Load online identities, initialize P2P endpoints for each identity, handle commands, manage protocol services");
}

async fn setup_protocol_handlers(
    _daemon_key: fastn_id52::SecretKey,
    _response_tx: broadcast::Sender<DaemonResponse>,
) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    todo!("Initialize and register protocol handlers (Echo, Shell) with P2P listener");
}