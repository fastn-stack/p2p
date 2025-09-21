/// Server builder for clean multi-protocol server setup
///
/// Also implements Future so you can .await on it to start the server
pub struct ServerBuilder {
    private_key: fastn_id52::SecretKey,
    request_handlers: std::collections::HashMap<serde_json::Value, RequestHandler>,
}

type RequestHandler = Box<
    dyn Fn(String) -> std::pin::Pin<Box<dyn std::future::Future<Output = String> + Send>>
        + Send
        + Sync,
>;

impl ServerBuilder {
    pub fn new(private_key: fastn_id52::SecretKey) -> Self {
        Self {
            private_key,
            request_handlers: std::collections::HashMap::new(),
        }
    }

    /// Add a request/response handler for a protocol
    pub fn handle_requests<P, F, Fut, INPUT, OUTPUT, ERROR>(mut self, protocol: P, handler: F) -> Self
    where
        P: serde::Serialize + std::fmt::Debug,
        F: Fn(INPUT) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<OUTPUT, ERROR>> + Send,
        INPUT: serde::de::DeserializeOwned,
        OUTPUT: serde::Serialize,
        ERROR: serde::Serialize + std::error::Error + Send + Sync + 'static,
    {
        // Convert protocol to JSON value for lookup
        let protocol_key = serde_json::to_value(&protocol)
            .expect("Protocol must be serializable");

        // Create a type-erased handler that works with JSON strings
        let boxed_handler: RequestHandler = {
            let handler = std::sync::Arc::new(handler);
            Box::new(move |request_json: String| {
                let handler = handler.clone();
                Box::pin(async move {
                    // Deserialize request
                    let input: INPUT = match serde_json::from_str(&request_json) {
                        Ok(input) => input,
                        Err(e) => {
                            let error_msg = format!("Failed to deserialize request: {}", e);
                            return serde_json::to_string(&error_msg).unwrap_or_else(|_| error_msg);
                        }
                    };

                    // Call handler
                    let result = handler(input).await;

                    // Serialize response (success or error)
                    match result {
                        Ok(output) => serde_json::to_string(&output)
                            .unwrap_or_else(|e| format!("Failed to serialize response: {}", e)),
                        Err(error) => serde_json::to_string(&error)
                            .unwrap_or_else(|e| format!("Failed to serialize error: {}", e)),
                    }
                })
            })
        };

        self.request_handlers.insert(protocol_key, boxed_handler);
        self
    }

    /// Add a streaming handler for a protocol
    pub fn handle_streams<P, F, Fut, DATA, STATE, ERROR>(self, _protocol: P, _state: STATE, _handler: F) -> Self
    where
        P: serde::Serialize + std::fmt::Debug,
        DATA: serde::de::DeserializeOwned,
        STATE: Clone + Send + Sync + 'static,
        F: Fn(crate::server::Session<P>, DATA, STATE) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<(), ERROR>> + Send,
        ERROR: std::error::Error + Send + Sync + 'static,
    {
        // TODO: Store the handler for protocol dispatch with automatic data extraction and state cloning
        self
    }
}

// Implement Future for ServerBuilder so it can be awaited
impl std::future::Future for ServerBuilder {
    type Output = Result<(), Box<dyn std::error::Error>>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        // We need to spawn the actual server task
        let this = self.get_mut();
        
        // Clone data for the spawned task
        let private_key = this.private_key.clone();
        let handlers = std::mem::take(&mut this.request_handlers);
        
        println!("🎧 Server listening on: {}", private_key.id52());
        
        // Spawn the server task
        crate::spawn(async move {
            if let Err(e) = run_server(private_key, handlers).await {
                tracing::error!("Server error: {}", e);
            }
        });
        
        // Return Ready to indicate the "startup" is complete
        // The actual server runs in the background task
        std::task::Poll::Ready(Ok(()))
    }
}

async fn run_server(
    private_key: fastn_id52::SecretKey,
    handlers: std::collections::HashMap<serde_json::Value, RequestHandler>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Get endpoint for listening
    let endpoint = fastn_net::get_endpoint(private_key).await?;
    
    // Wrap handlers in Arc for sharing across tasks
    let handlers = std::sync::Arc::new(handlers);
    
    loop {
        tokio::select! {
            _ = crate::cancelled() => {
                tracing::info!("Server shutting down");
                break;
            }
            conn = endpoint.accept() => {
                let conn = match conn {
                    Some(conn) => conn,
                    None => {
                        tracing::info!("No more connections");
                        break;
                    }
                };
                
                let handlers = handlers.clone();
                crate::spawn(async move {
                    if let Err(e) = handle_connection(conn, &handlers).await {
                        tracing::error!("Connection error: {}", e);
                    }
                });
            }
        }
    }
    
    Ok(())
}

async fn handle_connection(
    conn: iroh::endpoint::Incoming,
    handlers: &std::collections::HashMap<serde_json::Value, RequestHandler>,
) -> Result<(), Box<dyn std::error::Error>> {
    let conn = conn.await?;
    
    // Get peer's ID52 for logging and security
    let peer_key = fastn_net::get_remote_id52(&conn).await?;
    tracing::debug!("Connection established with peer: {}", peer_key.id52());
    
    loop {
        // Accept bidirectional stream - accept fastn-p2p protocol
        let (protocol, mut send_stream, mut recv_stream) = 
            fastn_net::accept_bi(&conn, &[fastn_net::Protocol::Generic(serde_json::Value::String("fastn-p2p".to_string()))]).await?;
            
        // Verify this is fastn-p2p protocol
        match protocol {
            fastn_net::Protocol::Generic(json) if json == serde_json::Value::String("fastn-p2p".to_string()) => {
                // Good, this is our protocol
            }
            other => {
                tracing::warn!("Unsupported protocol for request/response: {:?}", other);
                continue;
            }
        };
        
        // Read the wrapper request
        let wrapper_json = fastn_net::next_string(&mut recv_stream).await?;
        
        // Parse wrapper request to extract protocol and data
        let wrapper: serde_json::Value = match serde_json::from_str(&wrapper_json) {
            Ok(wrapper) => wrapper,
            Err(e) => {
                tracing::warn!("Failed to parse wrapper request: {}", e);
                let error_msg = format!("Failed to parse wrapper request: {}", e);
                send_stream.write_all(error_msg.as_bytes()).await?;
                send_stream.write_all(b"\n").await?;
                continue;
            }
        };
        
        // Extract protocol and data from wrapper
        let (protocol_json, data_json) = match (wrapper.get("protocol"), wrapper.get("data")) {
            (Some(protocol), Some(data)) => (protocol.clone(), data.clone()),
            _ => {
                tracing::warn!("Invalid wrapper request format: missing protocol or data");
                let error_msg = "Invalid wrapper request format: missing protocol or data";
                send_stream.write_all(error_msg.as_bytes()).await?;
                send_stream.write_all(b"\n").await?;
                continue;
            }
        };
        
        // Find handler for this protocol
        let handler = match handlers.get(&protocol_json) {
            Some(handler) => handler,
            None => {
                tracing::warn!("No handler for protocol {:?} from peer {}", protocol_json, peer_key.id52());
                let error_msg = format!("No handler for protocol: {:?}", protocol_json);
                send_stream.write_all(error_msg.as_bytes()).await?;
                send_stream.write_all(b"\n").await?;
                continue;
            }
        };
        
        tracing::debug!("Handling protocol {:?} from peer {}", protocol_json, peer_key.id52());
        
        // Convert data back to JSON string for handler
        let request_json = serde_json::to_string(&data_json).unwrap_or_else(|e| {
            format!("Failed to serialize data: {}", e)
        });
        
        // Call handler
        let response_json = handler(request_json).await;
        
        // Send response (with safety check in case handler tries to send multiple)
        match send_response(&mut send_stream, &response_json, &peer_key, &protocol_json).await {
            Ok(_) => {
                tracing::debug!("Successfully sent response to peer {}", peer_key.id52());
            }
            Err(e) => {
                tracing::error!("Failed to send response to peer {}: {}", peer_key.id52(), e);
            }
        }
        
        break; // One request per connection for now
    }
    
    Ok(())
}

/// Send response with proper error handling and logging
async fn send_response(
    send_stream: &mut iroh::endpoint::SendStream,
    response_json: &str,
    peer_key: &fastn_id52::PublicKey,
    protocol_json: &serde_json::Value,
) -> Result<(), Box<dyn std::error::Error>> {
    // Send JSON followed by newline (same format as original)
    send_stream.write_all(response_json.as_bytes()).await?;
    send_stream.write_all(b"\n").await?;
    
    tracing::trace!("Sent response for protocol {:?} to peer {}: {} bytes", 
                   protocol_json, peer_key.id52(), response_json.len());
    
    Ok(())
}

/// Start listening for P2P connections
///
/// Returns a ServerBuilder that you can add handlers to, then await to start the server
pub fn listen(private_key: fastn_id52::SecretKey) -> ServerBuilder {
    ServerBuilder::new(private_key)
}
