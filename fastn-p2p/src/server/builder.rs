/// Server builder for clean multi-protocol server setup
pub struct ServerBuilder {
    // TODO: Store protocol handlers here
}

impl ServerBuilder {
    /// Add more request/response handlers to this builder
    pub fn handle_requests<P, F, Fut, INPUT, OUTPUT, ERROR>(self, _protocol: P, _handler: F) -> Self 
    where
        P: serde::Serialize + std::fmt::Display,
        F: Fn(INPUT) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<OUTPUT, ERROR>> + Send,
        INPUT: serde::de::DeserializeOwned,
        OUTPUT: serde::Serialize,
        ERROR: serde::Serialize,
    {
        // TODO: Store the handler for protocol dispatch
        self
    }

    /// Add more streaming handlers to this builder
    pub fn handle_streams<P, F, Fut>(self, _protocol: P, _handler: F) -> Self
    where
        P: serde::Serialize + std::fmt::Display,
        F: Fn(crate::server::Session<P>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send,
    {
        // TODO: Store the handler for protocol dispatch
        self
    }

    /// Start listening for connections with the given private key
    pub async fn listen(self, private_key: fastn_id52::SecretKey) -> Result<(), Box<dyn std::error::Error>> {
        // TODO: Implement actual server listening logic
        // For now, use the existing listen mechanism as placeholder
        println!("🔧 Server builder listening on {} - TODO: implement protocol dispatch", private_key.id52());
        
        // Placeholder: just sleep to prevent immediate exit
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }
}

/// Start a server builder with a request/response handler
pub fn handle_requests<P, F, Fut, INPUT, OUTPUT, ERROR>(protocol: P, handler: F) -> ServerBuilder 
where
    P: serde::Serialize + std::fmt::Display,
    F: Fn(INPUT) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = Result<OUTPUT, ERROR>> + Send,
    INPUT: serde::de::DeserializeOwned,
    OUTPUT: serde::Serialize,
    ERROR: serde::Serialize,
{
    // TODO: Store the handler for protocol dispatch
    ServerBuilder {}
}

/// Start a server builder with a streaming handler
pub fn handle_streams<P, F, Fut>(protocol: P, handler: F) -> ServerBuilder
where
    P: serde::Serialize + std::fmt::Display,
    F: Fn(crate::server::Session<P>) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send,
{
    // TODO: Store the handler for protocol dispatch
    ServerBuilder {}
}

// Note: server() function removed - just chain handle_requests/handle_streams directly