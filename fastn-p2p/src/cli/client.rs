//! Client functionality for fastn-p2p CLI
//!
//! This module handles CLI client commands using the lightweight fastn-p2p-client crate.
//! It reads from stdin, communicates with the daemon, and outputs results.

use std::path::PathBuf;

/// Make a request/response call to a peer via the daemon
pub async fn call(
    _fastn_home: PathBuf,
    _peer: String,
    _protocol: String,
) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Parse peer ID, read JSON from stdin, use fastn_p2p_client::call()
    todo!("Read JSON from stdin, use fastn-p2p-client::call() to send via daemon, print response to stdout");
}

/// Open a bidirectional stream to a peer via the daemon  
pub async fn stream(
    _fastn_home: PathBuf,
    _peer: String,
    _protocol: String,
) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Parse peer ID, use fastn_p2p_client::connect(), pipe stdin/stdout
    todo!("Use fastn-p2p-client::connect() to establish stream via daemon, pipe stdin/stdout bidirectionally");
}