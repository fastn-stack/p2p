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
    // TODO: Implement streaming connection establishment with automatic data sending
    todo!(
        "Connect to {target} with protocol {protocol:?} and data, using {}",
        our_key.id52()
    )
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
    #[error("Failed to establish streaming connection: {source}")]
    Connection { source: eyre::Error },

    #[error("Stream error: {source}")]
    Stream { source: eyre::Error },
}
