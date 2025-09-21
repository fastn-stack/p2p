/// Client-side P2P communication
///
/// This module provides client APIs for establishing both simple request/response
/// connections and complex streaming sessions with remote P2P endpoints.

/// Simple request/response communication (existing functionality)
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

/// Establish streaming P2P session with automatic data sending
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

/// Client-side streaming session
pub struct Session {
    /// Input stream to server
    pub stdin: iroh::endpoint::SendStream,
    /// Output stream from server
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

    /// Copy from session stdout to a writer (download pattern)
    pub async fn copy_to<W>(&mut self, mut writer: W) -> std::io::Result<u64>
    where
        W: tokio::io::AsyncWrite + Unpin,
    {
        tokio::io::copy(&mut self.stdout, &mut writer).await
    }

    /// Copy from a reader to session stdin (upload pattern)
    pub async fn copy_from<R>(&mut self, mut reader: R) -> std::io::Result<u64>
    where
        R: tokio::io::AsyncRead + Unpin,
    {
        tokio::io::copy(&mut reader, &mut self.stdin).await
    }

    /// Bidirectional copy - copy reader to stdin and stdout to writer simultaneously
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
