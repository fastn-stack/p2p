//! Client functionality for fastn-p2p

use std::path::PathBuf;

/// Make a request/response call to a peer via the daemon
pub async fn call(
    fastn_home: PathBuf,
    peer: String,
    protocol: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let socket_path = fastn_home.join("control.sock");
    
    if !socket_path.exists() {
        eprintln!("❌ Daemon not running. Socket not found: {}", socket_path.display());
        eprintln!("   Start daemon with: fastn-p2p daemon");
        return Err("Daemon not available".into());
    }

    todo!("Connect to daemon via Unix socket, send JSON request with stdin data, print response to stdout")
}

/// Open a bidirectional stream to a peer via the daemon
pub async fn stream(
    fastn_home: PathBuf,
    peer: String,
    protocol: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let socket_path = fastn_home.join("control.sock");
    
    if !socket_path.exists() {
        eprintln!("❌ Daemon not running. Socket not found: {}", socket_path.display());
        eprintln!("   Start daemon with: fastn-p2p daemon");
        return Err("Daemon not available".into());
    }

    todo!("Connect to daemon via Unix socket, send JSON stream request, pipe stdin/stdout bidirectionally")
}