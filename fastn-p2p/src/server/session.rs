/// Server-side streaming session (handles both RPC and streaming)
pub struct Session<PROTOCOL> {
    /// Protocol negotiated with client
    pub protocol: PROTOCOL,
    /// Stream to client (stdout)
    pub send: iroh::endpoint::SendStream,
    /// Stream from client (stdin)  
    pub recv: iroh::endpoint::RecvStream,
    /// Peer's public key
    pub peer: fastn_id52::PublicKey,
    /// Context for this session (integration with fastn-context)
    pub context: std::sync::Arc<fastn_context::Context>,
}

impl<PROTOCOL> Session<PROTOCOL> {
    /// Get the peer's public key
    pub fn peer(&self) -> &fastn_id52::PublicKey {
        &self.peer
    }

    /// Get the context for this session
    pub fn context(&self) -> &std::sync::Arc<fastn_context::Context> {
        &self.context
    }

    /// Convert to Request for RPC handling (consumes Session)
    pub fn into_request(self) -> super::request::Request<PROTOCOL> {
        // TODO: Convert Session to Request for RPC pattern
        todo!("Convert Session to Request for RPC handling")
    }

    /// Open unidirectional stream back to client (e.g., stderr)
    pub async fn open_uni(
        &mut self,
    ) -> Result<iroh::endpoint::SendStream, fastn_p2p_client::ConnectionError> {
        // TODO: Open unidirectional stream to client
        todo!("Open unidirectional stream back to client")
    }

    /// Open bidirectional stream back to client
    pub async fn open_bi(
        &mut self,
    ) -> Result<
        (iroh::endpoint::SendStream, iroh::endpoint::RecvStream),
        fastn_p2p_client::ConnectionError,
    > {
        // TODO: Open bidirectional stream to client
        todo!("Open bidirectional stream back to client")
    }

    /// Copy from session recv stream to a writer (download pattern)
    pub async fn copy_to<W>(&mut self, mut writer: W) -> std::io::Result<u64>
    where
        W: tokio::io::AsyncWrite + Unpin,
    {
        tokio::io::copy(&mut self.recv, &mut writer).await
    }

    /// Copy from a reader to session send stream (upload pattern)
    pub async fn copy_from<R>(&mut self, mut reader: R) -> std::io::Result<u64>
    where
        R: tokio::io::AsyncRead + Unpin,
    {
        tokio::io::copy(&mut reader, &mut self.send).await
    }

    /// Bidirectional copy - copy reader to send stream and recv stream to writer simultaneously
    pub async fn copy_both<R, W>(
        &mut self,
        mut reader: R,
        mut writer: W,
    ) -> std::io::Result<(u64, u64)>
    where
        R: tokio::io::AsyncRead + Unpin,
        W: tokio::io::AsyncWrite + Unpin,
    {
        let to_remote = tokio::io::copy(&mut reader, &mut self.send);
        let from_remote = tokio::io::copy(&mut self.recv, &mut writer);

        futures_util::try_join!(to_remote, from_remote)
    }
}

/// Create a new Session (used internally by listener)
pub(crate) fn create_session<PROTOCOL>(
    protocol: PROTOCOL,
    send: iroh::endpoint::SendStream,
    recv: iroh::endpoint::RecvStream,
    peer: fastn_id52::PublicKey,
    parent_context: &std::sync::Arc<fastn_context::Context>,
) -> Session<PROTOCOL> {
    // Use parent context for now (can create child context later)
    Session {
        protocol,
        send,
        recv,
        peer,
        context: parent_context.clone(),
    }
}
