//! Client-side P2P communication via daemon
//!
//! This module provides the same API as the original fastn_p2p::client but
//! routes all communication through the fastn-p2p daemon via Unix socket.

use std::path::PathBuf;

use crate::error::{ClientError, ConnectionError};

/// Make a type-safe request/response call to a remote peer via daemon
///
/// This function connects to the local fastn-p2p daemon via Unix socket,
/// sends a request, and waits for a response. The API is identical to the
/// original direct P2P version but uses daemon coordination.
///
/// # Parameters
///
/// * `our_key` - Your private key (only used for daemon authentication)
/// * `target` - The public key of the peer to connect to
/// * `protocol` - The protocol enum variant for this request type
/// * `request` - The request data to send
///
/// # Returns
///
/// Returns a nested Result matching the original API:
/// - Outer `Result`: Network/daemon communication errors
/// - Inner `Result`: Application level success vs error
///
/// # Example (matches original examples)
///
/// ```rust,no_run
/// use fastn_p2p_client as fastn_p2p;
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
/// enum EchoProtocol { Echo }
///
/// #[derive(Serialize, Deserialize)]
/// struct EchoRequest { message: String }
///
/// #[derive(Serialize, Deserialize)]
/// struct EchoResponse { echoed: String }
///
/// #[derive(Serialize, Deserialize, thiserror::Error)]
/// #[error("Echo error")]
/// struct EchoError;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let our_key = fastn_p2p::SecretKey::generate();
/// let target = fastn_p2p::SecretKey::generate().public_key();
/// let request = EchoRequest { message: "Hello".to_string() };
///
/// let result: Result<EchoResponse, EchoError> = fastn_p2p::client::call(
///     our_key, target, EchoProtocol::Echo, request
/// ).await?;
/// # Ok(())
/// # }
/// ```
pub async fn call<PROTOCOL, REQUEST, RESPONSE, ERROR>(
    _our_key: fastn_id52::SecretKey,
    _target: fastn_id52::PublicKey,
    _protocol: PROTOCOL,
    _request: REQUEST,
) -> Result<Result<RESPONSE, ERROR>, ClientError>
where
    PROTOCOL: serde::Serialize + for<'de> serde::Deserialize<'de> + Clone + PartialEq + std::fmt::Debug + Send + Sync + 'static,
    REQUEST: serde::Serialize + for<'de> serde::Deserialize<'de>,
    RESPONSE: serde::Serialize + for<'de> serde::Deserialize<'de>,
    ERROR: serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    todo!("Connect to fastn-p2p daemon via Unix socket, send JSON call request, receive JSON response");
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