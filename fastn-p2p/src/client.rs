//! Client-side P2P communication
//!
//! This module provides client APIs for establishing both simple request/response
//! connections and complex streaming sessions with remote P2P endpoints.
//!
//! ## Request/Response Communication
//!
//! Use [`call`] for simple request/response patterns:
//!
//! ```rust,no_run
//! # use fastn_p2p::SecretKey;
//! # use serde::{Serialize, Deserialize};
//! #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
//! enum MyProtocol { Echo }
//!
//! #[derive(Serialize, Deserialize)]
//! struct MyRequest { message: String }
//!
//! #[derive(Serialize, Deserialize)]  
//! struct MyResponse { echoed: String }
//!
//! #[derive(Serialize, Deserialize, thiserror::Error)]
//! #[error("My error: {message}")]
//! struct MyError { message: String }
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let private_key = SecretKey::generate();
//! let target = SecretKey::generate().public_key();
//! let request = MyRequest { message: "Hello".to_string() };
//!
//! let result: Result<MyResponse, MyError> = fastn_p2p::client::call(
//!     private_key, target, MyProtocol::Echo, request
//! ).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Streaming Communication
//!
//! Use [`connect`] for streaming data between peers:
//!
//! ```rust,no_run
//! # use fastn_p2p::SecretKey;
//! # use serde::{Serialize, Deserialize};
//! #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
//! enum FileProtocol { Download }
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let private_key = SecretKey::generate();
//! let target = SecretKey::generate().public_key();
//!
//! // Connect and send filename as initial data
//! let mut session = fastn_p2p::client::connect(
//!     private_key, target, FileProtocol::Download, "file.txt"
//! ).await?;
//!
//! // Stream file content to local file
//! let mut local_file = tokio::fs::File::create("downloaded.txt").await?;
//! session.copy_to(&mut local_file).await?;
//! # Ok(())
//! # }
//! ```

/// Make a type-safe request/response call to a remote peer
///
/// This function establishes a connection to a remote peer, sends a request,
/// and waits for a response. The entire interaction is type-safe and handles
/// both successful responses and application-level errors.
///
/// # Parameters
///
/// * `our_key` - Your private key for authentication
/// * `target` - The public key of the peer to connect to
/// * `protocol` - The protocol enum variant for this request type
/// * `request` - The request data to send
///
/// # Returns
///
/// Returns a nested Result:
/// - Outer `Result`: Network/transport level errors ([`CallError`])
/// - Inner `Result`: Application level success (`RESPONSE`) vs error (`ERROR`)
///
/// # Example
///
/// ```rust,no_run
/// # use fastn_p2p::{SecretKey, client};
/// # use serde::{Serialize, Deserialize};
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
/// #[error("Echo failed: {reason}")]
/// struct EchoError { reason: String }
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let our_key = SecretKey::generate();
/// let target = SecretKey::generate().public_key();
/// let request = EchoRequest { message: "Hello P2P!".to_string() };
///
/// let result: Result<EchoResponse, EchoError> = client::call(
///     our_key, target, EchoProtocol::Echo, request
/// ).await?;
///
/// match result {
///     Ok(response) => println!("Got response: {}", response.echoed),
///     Err(app_error) => println!("Application error: {}", app_error),
/// }
/// # Ok(())
/// # }
/// ```
pub async fn call<PROTOCOL, REQUEST, RESPONSE, ERROR>(
    our_key: fastn_id52::SecretKey,
    target: fastn_id52::PublicKey,
    protocol: PROTOCOL,
    request: REQUEST,
) -> Result<Result<RESPONSE, ERROR>, CallError>
where
    PROTOCOL: serde::Serialize
        + for<'de> serde::Deserialize<'de>
        + Clone
        + PartialEq
        + std::fmt::Debug
        + Send
        + Sync
        + 'static,
    REQUEST: serde::Serialize + for<'de> serde::Deserialize<'de>,
    RESPONSE: serde::Serialize + for<'de> serde::Deserialize<'de>,
    ERROR: serde::Serialize + for<'de> serde::Deserialize<'de>,
{
    // Delegate to existing coordination infrastructure (will be restored in Phase 5)
    crate::coordination::internal_call(our_key, &target, protocol, request).await
}

/// Establish a streaming P2P session with automatic data sending
///
/// This function creates a persistent bidirectional connection to a remote peer
/// for streaming data. The initial `data` parameter is automatically sent as
/// part of the connection handshake, allowing the server to understand what
/// the client wants to stream.
///
/// # Parameters
///
/// * `our_key` - Your private key for authentication
/// * `target` - The public key of the peer to connect to  
/// * `protocol` - The protocol enum variant for this stream type
/// * `data` - Initial data sent with the connection (e.g., filename, request details)
///
/// # Returns
///
/// Returns a [`Session`] that provides methods for streaming data to/from the peer.
///
/// # Example
///
/// ```rust,no_run
/// # use fastn_p2p::{SecretKey, client};
/// # use serde::{Serialize, Deserialize};
/// #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
/// enum FileProtocol { Download }
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let our_key = SecretKey::generate();
/// let target = SecretKey::generate().public_key();
/// let filename = "data.txt";
///
/// // Connect and automatically send filename to server
/// let mut session = client::connect(
///     our_key, target, FileProtocol::Download, filename
/// ).await?;
///
/// // Stream the file content to a local file
/// let mut output = tokio::fs::File::create("downloaded_data.txt").await?;
/// let bytes_copied = session.copy_to(&mut output).await?;
/// println!("Downloaded {} bytes", bytes_copied);
/// # Ok(())
/// # }
/// ```
pub async fn connect<PROTOCOL, DATA>(
    our_key: fastn_id52::SecretKey,
    target: fastn_id52::PublicKey,
    protocol: PROTOCOL,
    data: DATA,
) -> Result<Session, ConnectionError>
where
    PROTOCOL: serde::Serialize + for<'de> serde::Deserialize<'de> + std::fmt::Debug,
    DATA: serde::Serialize,
{
    // Get endpoint for the sender
    let endpoint = fastn_net::get_endpoint(our_key.clone())
        .await
        .map_err(|source| ConnectionError::Endpoint { source })?;

    // Connect to target
    let target_node_id = iroh::NodeId::from(
        iroh::PublicKey::from_bytes(&target.to_bytes())
            .map_err(|e| ConnectionError::Stream { source: eyre::Error::from(e) })?
    );
    let conn = endpoint.connect(target_node_id, &fastn_net::APNS_IDENTITY)
        .await
        .map_err(|e| ConnectionError::Stream { source: eyre::Error::from(e) })?;
    
    // Send handshake first
    let handshake_protocol = fastn_net::Protocol::Generic(
        serde_json::Value::String(crate::handshake::HANDSHAKE_PROTOCOL.to_string())
    );
    
    let (mut hs_send, mut hs_recv) = conn.open_bi().await
        .map_err(|e| ConnectionError::Stream { source: eyre::Error::from(e) })?;
    
    // Send handshake protocol identifier
    let protocol_json = serde_json::to_string(&handshake_protocol)
        .map_err(|source| ConnectionError::Serialization { source })?;
    hs_send.write_all(protocol_json.as_bytes()).await
        .map_err(|e| ConnectionError::Send { source: eyre::Error::from(e) })?;
    hs_send.write_all(b"\n").await
        .map_err(|e| ConnectionError::Send { source: eyre::Error::from(e) })?;
    
    // Wait for ACK
    let ack = fastn_net::next_string(&mut hs_recv).await
        .map_err(|source| ConnectionError::Receive { source })?;
    if ack != fastn_net::ACK {
        return Err(ConnectionError::Receive { 
            source: eyre::anyhow!("Expected ACK, got: {}", ack) 
        });
    }
    
    // Send ClientHello
    let client_hello = crate::handshake::ClientHello::new(
        "fastn-p2p-client",
        env!("CARGO_PKG_VERSION")
    ).with_protocol(&protocol);
    
    let hello_json = serde_json::to_string(&client_hello)
        .map_err(|source| ConnectionError::Serialization { source })?;
    hs_send.write_all(hello_json.as_bytes()).await
        .map_err(|e| ConnectionError::Send { source: eyre::Error::from(e) })?;
    hs_send.write_all(b"\n").await
        .map_err(|e| ConnectionError::Send { source: eyre::Error::from(e) })?;
    
    // Read ServerHello
    let server_hello: crate::handshake::ServerHello = fastn_net::next_json(&mut hs_recv).await
        .map_err(|source| ConnectionError::Receive { source })?;
    
    // Check if handshake succeeded
    let accepted_protocols = match server_hello {
        crate::handshake::ServerHello::Success { 
            accepted_protocols, .. 
        } => accepted_protocols,
        crate::handshake::ServerHello::Failure { code } => {
            return Err(ConnectionError::Receive { 
                source: eyre::anyhow!("Server rejected handshake: {:?}", code)
            });
        }
    };
    
    // Check if our protocol is accepted
    let protocol_json = serde_json::to_value(&protocol)
        .map_err(|e| ConnectionError::Serialization { source: e })?;
    if !accepted_protocols.contains(&protocol_json) {
        return Err(ConnectionError::Receive { 
            source: eyre::anyhow!("Server doesn't support requested protocol")
        });
    }
    
    hs_send.finish()
        .map_err(|e| ConnectionError::Send { source: eyre::Error::from(e) })?;
    
    // Now open the actual application protocol stream
    let app_protocol = fastn_net::Protocol::Generic(serde_json::Value::String("fastn-p2p".to_string()));
    
    let (mut send_stream, mut recv_stream) = conn.open_bi().await
        .map_err(|e| ConnectionError::Stream { source: eyre::Error::from(e) })?;
    
    // Send app protocol identifier  
    let app_protocol_json = serde_json::to_string(&app_protocol)
        .map_err(|source| ConnectionError::Serialization { source })?;
    send_stream.write_all(app_protocol_json.as_bytes()).await
        .map_err(|e| ConnectionError::Send { source: eyre::Error::from(e) })?;
    send_stream.write_all(b"\n").await
        .map_err(|e| ConnectionError::Send { source: eyre::Error::from(e) })?;
    
    // Wait for ACK
    let ack = fastn_net::next_string(&mut recv_stream).await
        .map_err(|source| ConnectionError::Receive { source })?;
    if ack != fastn_net::ACK {
        return Err(ConnectionError::Receive { 
            source: eyre::anyhow!("Expected ACK for app protocol, got: {}", ack) 
        });
    }

    // Convert user protocol to JSON for embedding in request
    let protocol_json =
        serde_json::to_value(&protocol).map_err(|e| ConnectionError::Serialization { source: e })?;

    // Create wrapper request with protocol and data
    let wrapper_request = serde_json::json!({
        "protocol": protocol_json,
        "data": data
    });
    let request_json = serde_json::to_string(&wrapper_request)
        .map_err(|source| ConnectionError::Serialization { source })?;

    // Send JSON followed by newline
    send_stream
        .write_all(request_json.as_bytes())
        .await
        .map_err(|e| ConnectionError::Send {
            source: eyre::Error::from(e),
        })?;
    send_stream
        .write_all(b"\n")
        .await
        .map_err(|e| ConnectionError::Send {
            source: eyre::Error::from(e),
        })?;

    // Create and return the Session
    Ok(Session {
        stdin: send_stream,
        stdout: recv_stream,
    })
}

/// Client-side streaming session for bidirectional P2P communication
///
/// A `Session` represents an active streaming connection to a remote peer.
/// It provides high-level methods for copying data to/from the peer,
/// abstracting away the underlying stream management.
///
/// # Usage Patterns
///
/// ## Download Pattern
/// Use [`copy_to`] to stream data from the peer to a local writer:
/// ```rust,no_run
/// # use fastn_p2p::client::Session;
/// # async fn example(mut session: Session) -> std::io::Result<()> {
/// let mut file = tokio::fs::File::create("download.txt").await?;
/// let bytes = session.copy_to(&mut file).await?;
/// println!("Downloaded {} bytes", bytes);
/// # Ok(())
/// # }
/// ```
///
/// ## Upload Pattern  
/// Use [`copy_from`] to stream data from a local reader to the peer:
/// ```rust,no_run
/// # use fastn_p2p::client::Session;
/// # async fn example(mut session: Session) -> std::io::Result<()> {
/// let mut file = tokio::fs::File::open("upload.txt").await?;
/// let bytes = session.copy_from(&mut file).await?;
/// println!("Uploaded {} bytes", bytes);
/// # Ok(())
/// # }
/// ```
///
/// ## Bidirectional Pattern
/// Use [`copy_both`] for simultaneous bidirectional streaming:
/// ```rust,no_run
/// # use fastn_p2p::client::Session;
/// # async fn example(mut session: Session) -> std::io::Result<()> {
/// let input = tokio::fs::File::open("input.txt").await?;
/// let mut output = tokio::fs::File::create("output.txt").await?;
/// let (sent, received) = session.copy_both(input, &mut output).await?;
/// println!("Sent {} bytes, received {} bytes", sent, received);
/// # Ok(())
/// # }
/// ```
///
/// [`copy_to`]: Self::copy_to
/// [`copy_from`]: Self::copy_from
/// [`copy_both`]: Self::copy_both
pub struct Session {
    /// Input stream to server (for sending data to peer)
    pub stdin: iroh::endpoint::SendStream,
    /// Output stream from server (for receiving data from peer)
    pub stdout: iroh::endpoint::RecvStream,
    // TODO: Add context integration
    // context: std::sync::Arc<fastn_context::Context>,
}

impl Session {
    /// Accept unidirectional stream back from server (e.g., stderr)
    pub async fn accept_uni(&mut self) -> Result<iroh::endpoint::RecvStream, ConnectionError> {
        // TODO: Accept incoming unidirectional stream from server
        todo!("Accept unidirectional stream from server")
    }

    /// Accept bidirectional stream back from server
    pub async fn accept_bi(
        &mut self,
    ) -> Result<(iroh::endpoint::RecvStream, iroh::endpoint::SendStream), ConnectionError> {
        // TODO: Accept incoming bidirectional stream from server
        todo!("Accept bidirectional stream from server")
    }

    /// Copy data from the peer to a local writer (download pattern)
    ///
    /// This method streams all data from the peer's output stream to the provided writer.
    /// It's equivalent to `tokio::io::copy(&mut session.stdout, &mut writer)` but
    /// provides a cleaner API.
    ///
    /// # Parameters
    /// * `writer` - Any type implementing [`tokio::io::AsyncWrite`]
    ///
    /// # Returns
    /// Returns the number of bytes copied on success.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use fastn_p2p::client::Session;
    /// # async fn example(mut session: Session) -> std::io::Result<()> {
    /// // Download to a file
    /// let mut file = tokio::fs::File::create("downloaded.txt").await?;
    /// let bytes = session.copy_to(&mut file).await?;
    /// println!("Downloaded {} bytes to file", bytes);
    ///
    /// // Or download to memory
    /// let mut buffer = Vec::new();
    /// let bytes = session.copy_to(&mut buffer).await?;
    /// println!("Downloaded {} bytes to memory", bytes);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn copy_to<W>(&mut self, mut writer: W) -> std::io::Result<u64>
    where
        W: tokio::io::AsyncWrite + Unpin,
    {
        tokio::io::copy(&mut self.stdout, &mut writer).await
    }

    /// Copy data from a local reader to the peer (upload pattern)
    ///
    /// This method streams all data from the provided reader to the peer's input stream.
    /// It's equivalent to `tokio::io::copy(&mut reader, &mut session.stdin)` but
    /// provides a cleaner API.
    ///
    /// # Parameters
    /// * `reader` - Any type implementing [`tokio::io::AsyncRead`]
    ///
    /// # Returns
    /// Returns the number of bytes copied on success.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use fastn_p2p::client::Session;
    /// # async fn example(mut session: Session) -> std::io::Result<()> {
    /// // Upload from a file
    /// let mut file = tokio::fs::File::open("upload.txt").await?;
    /// let bytes = session.copy_from(&mut file).await?;
    /// println!("Uploaded {} bytes from file", bytes);
    ///
    /// // Or upload from memory
    /// let data = b"Hello, peer!";
    /// let bytes = session.copy_from(&data[..]).await?;
    /// println!("Uploaded {} bytes from memory", bytes);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn copy_from<R>(&mut self, mut reader: R) -> std::io::Result<u64>
    where
        R: tokio::io::AsyncRead + Unpin,
    {
        tokio::io::copy(&mut reader, &mut self.stdin).await
    }

    /// Simultaneously copy data in both directions (bidirectional pattern)
    ///
    /// This method runs two copy operations concurrently:
    /// - Copies from `reader` to the peer (upload)
    /// - Copies from the peer to `writer` (download)
    ///
    /// This is useful for interactive protocols where you need to send and receive
    /// data at the same time, such as shell sessions or real-time communication.
    ///
    /// # Parameters
    /// * `reader` - Source to upload data from (implements [`tokio::io::AsyncRead`])
    /// * `writer` - Destination to download data to (implements [`tokio::io::AsyncWrite`])
    ///
    /// # Returns
    /// Returns a tuple of `(uploaded_bytes, downloaded_bytes)` on success.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use fastn_p2p::client::Session;
    /// # async fn example(mut session: Session) -> std::io::Result<()> {
    /// let input_file = tokio::fs::File::open("input.txt").await?;
    /// let mut output_file = tokio::fs::File::create("output.txt").await?;
    ///
    /// let (sent, received) = session.copy_both(input_file, &mut output_file).await?;
    /// println!("Sent {} bytes, received {} bytes", sent, received);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn copy_both<R, W>(
        &mut self,
        mut reader: R,
        mut writer: W,
    ) -> std::io::Result<(u64, u64)>
    where
        R: tokio::io::AsyncRead + Unpin,
        W: tokio::io::AsyncWrite + Unpin,
    {
        let to_remote = tokio::io::copy(&mut reader, &mut self.stdin);
        let from_remote = tokio::io::copy(&mut self.stdout, &mut writer);

        futures_util::try_join!(to_remote, from_remote)
    }
}

/// Errors for client operations
#[derive(Debug, thiserror::Error)]
pub enum CallError {
    #[error("Connection failed: {source}")]
    Connection { source: eyre::Error },

    #[error("Request/response error: {source}")]
    RequestResponse { source: eyre::Error },

    #[error("Serialization error: {source}")]
    Serialization { source: serde_json::Error },

    #[error("Endpoint error: {source}")]
    Endpoint { source: eyre::Error },

    #[error("Stream error: {source}")]
    Stream { source: eyre::Error },

    #[error("Send error: {source}")]
    Send { source: eyre::Error },

    #[error("Receive error: {source}")]
    Receive { source: eyre::Error },

    #[error("Deserialization error: {source}")]
    Deserialization { source: serde_json::Error },
}

#[derive(Debug, thiserror::Error)]
pub enum ConnectionError {
    #[error("Failed to get endpoint: {source}")]
    Endpoint { source: eyre::Error },
    
    #[error("Failed to establish streaming connection: {source}")]
    Connection { source: eyre::Error },

    #[error("Stream error: {source}")]
    Stream { source: eyre::Error },
    
    #[error("Serialization error: {source}")]
    Serialization { source: serde_json::Error },
    
    #[error("Failed to send data: {source}")]
    Send { source: eyre::Error },
    
    #[error("Failed to receive data: {source}")]
    Receive { source: eyre::Error },
}
