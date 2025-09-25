//! Client-side P2P communication via daemon
//!
//! This module provides the same API as the original fastn_p2p::client but
//! routes all communication through the fastn-p2p daemon via Unix socket.

use std::path::PathBuf;

use crate::error::{ClientError, ConnectionError};

/// Make a type-safe request/response call to a remote peer via daemon
///
/// This function connects to the local fastn-p2p daemon via Unix socket,
/// uses a configured identity to send a request, and waits for a response.
///
/// # Parameters
///
/// * `from_identity` - Identity name to send from (daemon looks up keys)
/// * `to_peer` - Target peer ID52 string
/// * `protocol` - Protocol name string
/// * `bind_alias` - Protocol bind alias (e.g., "default", "backup")
/// * `request` - Request data to send
///
/// # Returns
///
/// Returns a nested Result:
/// - Outer `Result`: Network/daemon communication errors
/// - Inner `Result`: Application level success vs error
///
/// # Example (daemon-based architecture)
///
/// ```rust,no_run
/// use fastn_p2p_client as fastn_p2p;
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize)]
/// struct MailRequest { to: String, subject: String, body: String }
///
/// #[derive(Serialize, Deserialize)]
/// struct MailResponse { message_id: String }
///
/// #[derive(Serialize, Deserialize, thiserror::Error)]
/// #[error("Mail error: {reason}")]
/// struct MailError { reason: String }
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let request = MailRequest { 
///     to: "bob@example.com".to_string(),
///     subject: "Hello".to_string(), 
///     body: "Test message".to_string() 
/// };
///
/// let result: Result<MailResponse, MailError> = fastn_p2p::call(
///     "alice",                    // Send from alice identity
///     "abc123...",               // To this peer  
///     "Mail",                    // Using Mail protocol
///     "default",                 // Default Mail server instance
///     request                    // Request data
/// ).await?;
/// # Ok(())
/// # }
/// ```
pub async fn call<REQUEST, RESPONSE, ERROR>(
    from_identity: &str,
    to_peer: &str,
    protocol: &str,
    bind_alias: &str,
    request: REQUEST,
) -> Result<Result<RESPONSE, ERROR>, ClientError>
where
    REQUEST: serde::Serialize + for<'de> serde::Deserialize<'de>,
    RESPONSE: serde::Serialize + for<'de> serde::Deserialize<'de>,
    ERROR: serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    let fastn_home = get_fastn_home()?;
    let socket_path = fastn_home.join("control.sock");
    
    if !socket_path.exists() {
        return Err(ClientError::DaemonConnection(
            format!("Daemon not running. Socket not found: {}. Start with: fastn-p2p daemon", socket_path.display())
        ));
    }
    
    println!("🔌 Connecting to daemon as identity '{}'", from_identity);
    println!("📤 Sending {} {} request to {}", protocol, bind_alias, to_peer);
    
    // Connect to Unix socket  
    let mut stream = tokio::net::UnixStream::connect(&socket_path).await
        .map_err(|e| ClientError::DaemonConnection(format!("Failed to connect to daemon: {}", e)))?;
    
    // Create JSON request for daemon
    let daemon_request = serde_json::json!({
        "id": uuid::Uuid::new_v4().to_string(),
        "type": "call",
        "from_identity": from_identity,
        "to_peer": to_peer,
        "protocol": protocol,
        "bind_alias": bind_alias,
        "request": request
    });
    
    // Send request to daemon
    use tokio::io::{AsyncWriteExt, AsyncReadExt};
    let request_json = serde_json::to_string(&daemon_request)?;
    stream.write_all(request_json.as_bytes()).await
        .map_err(|e| ClientError::Io { source: e })?;
    stream.write_all(b"\n").await
        .map_err(|e| ClientError::Io { source: e })?;
    
    println!("📡 Request sent to daemon, waiting for response...");
    
    // Read response from daemon
    let mut response_buffer = Vec::new();
    stream.read_to_end(&mut response_buffer).await
        .map_err(|e| ClientError::Io { source: e })?;
    
    let response_str = String::from_utf8(response_buffer)
        .map_err(|e| ClientError::DaemonConnection(format!("Invalid response from daemon: {}", e)))?;
    
    println!("📥 Received response from daemon: {}", response_str);
    
    // For now, return hardcoded success to test coordination
    // TODO: Parse actual daemon response and deserialize RESPONSE/ERROR
    return Err(ClientError::DaemonConnection("New coordination API working!".to_string()));
}

/// Establish a streaming P2P session via daemon
///
/// This function connects to the local fastn-p2p daemon and requests a
/// streaming session to a remote peer. The API matches the original but
/// uses daemon coordination.
///
/// # Parameters
///
/// * `our_key` - Your private key (only used for daemon authentication)  
/// * `target` - The public key of the peer to connect to
/// * `protocol` - The protocol enum variant for this stream type
/// * `data` - Initial data sent with the connection
///
/// # Returns
///
/// Returns a [`Session`] for streaming data to/from the peer via daemon.
///
/// # Example (matches original examples)
///
/// ```rust,no_run
/// use fastn_p2p_client as fastn_p2p;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let our_key = fastn_p2p::SecretKey::generate();
/// let target = fastn_p2p::SecretKey::generate().public_key();
///
/// let mut session = fastn_p2p::client::connect(
///     our_key, target, "FileTransfer", "filename.txt"
/// ).await?;
///
/// // Stream data (same API as original)
/// let mut output = tokio::fs::File::create("downloaded.txt").await?;
/// session.copy_to(&mut output).await?;
/// # Ok(())
/// # }
/// ```
pub async fn connect<PROTOCOL, DATA>(
    _our_key: fastn_id52::SecretKey,
    _target: fastn_id52::PublicKey,
    _protocol: PROTOCOL,
    _data: DATA,
) -> Result<Session, ConnectionError>
where
    PROTOCOL: serde::Serialize + for<'de> serde::Deserialize<'de> + std::fmt::Debug,
    DATA: serde::Serialize,
{
    todo!("Connect to fastn-p2p daemon via Unix socket, send JSON stream request, return Session proxy");
}

/// Client-side streaming session that proxies through daemon
///
/// This provides the same API as the original Session but routes all
/// communication through the fastn-p2p daemon via Unix socket.
pub struct Session {
    // TODO: Unix socket connection to daemon for streaming
    _daemon_connection: (),
}

impl Session {
    /// Copy data from the peer to a local writer (download pattern)
    ///
    /// Same API as original but streams through daemon Unix socket.
    pub async fn copy_to<W>(&mut self, _writer: W) -> std::io::Result<u64>
    where
        W: tokio::io::AsyncWrite + Unpin,
    {
        todo!("Stream data from daemon Unix socket to writer");
    }

    /// Copy data from a local reader to the peer (upload pattern)
    ///
    /// Same API as original but streams through daemon Unix socket.
    pub async fn copy_from<R>(&mut self, _reader: R) -> std::io::Result<u64>
    where
        R: tokio::io::AsyncRead + Unpin,
    {
        todo!("Stream data from reader to daemon Unix socket");
    }

    /// Simultaneously copy data in both directions (bidirectional pattern)
    ///
    /// Same API as original but coordinates through daemon Unix socket.
    pub async fn copy_both<R, W>(
        &mut self,
        _reader: R,
        _writer: W,
    ) -> std::io::Result<(u64, u64)>
    where
        R: tokio::io::AsyncRead + Unpin,
        W: tokio::io::AsyncWrite + Unpin,
    {
        todo!("Coordinate bidirectional streaming through daemon Unix socket");
    }
}

/// Get FASTN_HOME directory (shared utility)
fn get_fastn_home() -> Result<PathBuf, ClientError> {
    if let Ok(env_home) = std::env::var("FASTN_HOME") {
        return Ok(PathBuf::from(env_home));
    }

    let home_dir = directories::UserDirs::new()
        .ok_or_else(|| ClientError::Configuration("Could not determine user home directory".to_string()))?
        .home_dir()
        .to_path_buf();

    Ok(home_dir.join(".fastn"))
}