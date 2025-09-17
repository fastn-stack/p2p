/// Server builder for clean multi-protocol server setup
/// 
/// Also implements Future so you can .await on it to start the server
pub struct ServerBuilder {
    private_key: fastn_id52::SecretKey,
    // TODO: Store protocol handlers here
}

impl ServerBuilder {
    pub fn new(private_key: fastn_id52::SecretKey) -> Self {
        Self { private_key }
    }

    /// Add a request/response handler for a protocol
    pub fn handle_requests<P, F, Fut, INPUT, OUTPUT, ERROR>(self, _protocol: P, _handler: F) -> Self 
    where
        P: serde::Serialize + std::fmt::Debug,
        F: Fn(INPUT) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<OUTPUT, ERROR>> + Send,
        INPUT: serde::de::DeserializeOwned,
        OUTPUT: serde::Serialize,
        ERROR: serde::Serialize,
    {
        // TODO: Store the handler for protocol dispatch
        self
    }

    /// Add a streaming handler for a protocol
    pub fn handle_streams<P, F, Fut>(self, _protocol: P, _handler: F) -> Self
    where
        P: serde::Serialize + std::fmt::Debug,
        F: Fn(crate::server::Session<P>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send,
    {
        // TODO: Store the handler for protocol dispatch
        self
    }
}

// Implement Future for ServerBuilder so it can be awaited
impl std::future::Future for ServerBuilder {
    type Output = Result<(), Box<dyn std::error::Error>>;

    fn poll(self: std::pin::Pin<&mut Self>, _cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        // TODO: Implement actual server listening logic
        // For now, return a placeholder
        println!("🔧 Server listening on {} - TODO: implement protocol dispatch", self.private_key.id52());
        
        // Return Pending to simulate ongoing server (in real impl, this would manage the server loop)
        std::task::Poll::Pending
    }
}

/// Start listening for P2P connections
/// 
/// Returns a ServerBuilder that you can add handlers to, then await to start the server
pub fn listen(private_key: fastn_id52::SecretKey) -> ServerBuilder {
    ServerBuilder::new(private_key)
}