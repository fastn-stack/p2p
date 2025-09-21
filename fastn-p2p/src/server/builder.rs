/// Server builder for clean multi-protocol server setup
///
/// Also implements Future so you can .await on it to start the server
pub struct ServerBuilder {
    private_key: fastn_id52::SecretKey,
    request_handlers: std::collections::HashMap<serde_json::Value, RequestHandler>,
    stream_handlers: std::collections::HashMap<serde_json::Value, StreamHandler>,
    server_task: Option<std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), Box<dyn std::error::Error>>> + Send>>>,
}

type RequestHandler = Box<
    dyn Fn(String) -> std::pin::Pin<Box<dyn std::future::Future<Output = String> + Send>>
        + Send
        + Sync,
>;

type StreamHandler = Box<
    dyn Fn(
        iroh::endpoint::SendStream,
        iroh::endpoint::RecvStream,
        fastn_id52::PublicKey,
        String,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), Box<dyn std::error::Error>>> + Send>>
        + Send
        + Sync,
>;

impl ServerBuilder {
    pub fn new(private_key: fastn_id52::SecretKey) -> Self {
        Self {
            private_key,
            request_handlers: std::collections::HashMap::new(),
            stream_handlers: std::collections::HashMap::new(),
            server_task: None,
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
    pub fn handle_streams<P, F, Fut, DATA, STATE, ERROR>(mut self, protocol: P, state: STATE, handler: F) -> Self
    where
        P: serde::Serialize + for<'de> serde::Deserialize<'de> + std::fmt::Debug + Clone + Send + Sync + 'static,
        DATA: serde::de::DeserializeOwned + Send + 'static,
        STATE: Clone + Send + Sync + 'static,
        F: Fn(crate::server::Session<P>, DATA, STATE) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<(), ERROR>> + Send + 'static,
        ERROR: std::error::Error + Send + Sync + 'static,
    {
        // Convert protocol to JSON value for lookup
        let protocol_key = serde_json::to_value(&protocol)
            .expect("Protocol must be serializable");

        // Create a type-erased stream handler
        let boxed_handler: StreamHandler = {
            let handler = std::sync::Arc::new(handler);
            let state = std::sync::Arc::new(state);
            let protocol = protocol.clone();
            Box::new(move |send, recv, peer, data_json: String| {
                let handler = handler.clone();
                let state = state.clone();
                let protocol = protocol.clone();
                Box::pin(async move {
                    // Deserialize the initial data
                    let data: DATA = match serde_json::from_str(&data_json) {
                        Ok(data) => data,
                        Err(e) => {
                            return Err(Box::new(e) as Box<dyn std::error::Error>);
                        }
                    };
                    
                    // Create the session
                    let session = crate::server::Session {
                        protocol: protocol.clone(),
                        send,
                        recv,
                        peer,
                        context: fastn_context::Context::new("stream"),
                    };
                    
                    // Call the handler with session, data, and state
                    match handler(session, data, (*state).clone()).await {
                        Ok(()) => Ok(()),
                        Err(e) => Err(Box::new(e) as Box<dyn std::error::Error>),
                    }
                })
            })
        };

        self.stream_handlers.insert(protocol_key, boxed_handler);
        self
    }
}

// Implement Future for ServerBuilder so it can be awaited
impl std::future::Future for ServerBuilder {
    type Output = Result<(), Box<dyn std::error::Error>>;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        // If we haven't created the server task yet, create it
        if self.server_task.is_none() {
            let private_key = self.private_key.clone();
            let request_handlers = std::mem::take(&mut self.request_handlers);
            let stream_handlers = std::mem::take(&mut self.stream_handlers);
            
            println!("🎧 Server listening on: {}", private_key.id52());
            
            // Create the server future
            self.server_task = Some(Box::pin(run_server(private_key, request_handlers, stream_handlers)));
        }
        
        // Poll the server task
        if let Some(ref mut task) = self.server_task {
            std::pin::Pin::new(task).poll(cx)
        } else {
            unreachable!("server_task should be set above")
        }
    }
}

async fn run_server(
    private_key: fastn_id52::SecretKey,
    request_handlers: std::collections::HashMap<serde_json::Value, RequestHandler>,
    stream_handlers: std::collections::HashMap<serde_json::Value, StreamHandler>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Get endpoint for listening
    let endpoint = fastn_net::get_endpoint(private_key).await?;
    
    // Wrap handlers in Arc for sharing across tasks
    let request_handlers = std::sync::Arc::new(request_handlers);
    let stream_handlers = std::sync::Arc::new(stream_handlers);
    
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
                
                let request_handlers = request_handlers.clone();
                let stream_handlers = stream_handlers.clone();
                crate::spawn(async move {
                    if let Err(e) = handle_connection(conn, &request_handlers, &stream_handlers).await {
                        tracing::error!("Connection error: {}", e);
                    }
                });
            }
        }
    }
    
    Ok(())
}

// Structure of the wrapper request sent by client
#[derive(serde::Deserialize)]
struct WrapperRequest {
    protocol: serde_json::Value,
    data: serde_json::Value,
}

async fn handle_connection(
    conn: iroh::endpoint::Incoming,
    request_handlers: &std::collections::HashMap<serde_json::Value, RequestHandler>,
    stream_handlers: &std::collections::HashMap<serde_json::Value, StreamHandler>,
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
        
        // Read and parse the wrapper request directly as typed struct
        let wrapper: WrapperRequest = match fastn_net::next_json(&mut recv_stream).await {
            Ok(wrapper) => wrapper,
            Err(e) => {
                tracing::warn!("Failed to read/parse wrapper request: {}", e);
                let error_msg = format!("Failed to parse wrapper request: {}", e);
                send_stream.write_all(error_msg.as_bytes()).await?;
                send_stream.write_all(b"\n").await?;
                continue;
            }
        };
        
        // Check if it's a streaming or request handler
        let is_streaming = stream_handlers.contains_key(&wrapper.protocol);
        let is_request = request_handlers.contains_key(&wrapper.protocol);
        
        if !is_streaming && !is_request {
            tracing::warn!("No handler for protocol {:?} from peer {}", wrapper.protocol, peer_key.id52());
            let error_msg = format!("No handler for protocol: {:?}", wrapper.protocol);
            send_stream.write_all(error_msg.as_bytes()).await?;
            send_stream.write_all(b"\n").await?;
            continue;
        }
        
        // Convert data back to JSON string
        let data_json = serde_json::to_string(&wrapper.data).unwrap_or_else(|e| {
            format!("Failed to serialize data: {}", e)
        });
        
        if is_streaming {
            // Handle streaming protocol
            let handler = stream_handlers.get(&wrapper.protocol).unwrap();
            
            // Call the streaming handler with the streams
            match handler(send_stream, recv_stream, peer_key.clone(), data_json).await {
                Ok(()) => {
                    // Streaming completed successfully
                }
                Err(e) => {
                    tracing::error!("Streaming handler error: {}", e);
                }
            }
            // For streaming, the handler manages the streams, so we're done
        } else {
            // Handle request/response protocol
            let handler = request_handlers.get(&wrapper.protocol).unwrap();
            
            let response_json = handler(data_json).await;
            
            // Send response
            match send_response(&mut send_stream, &response_json, &peer_key, &wrapper.protocol).await {
                Ok(_) => {
                    // Response sent successfully
                }
                Err(e) => {
                    tracing::error!("Failed to send response to peer {}: {}", peer_key.id52(), e);
                    break;
                }
            }
            
            // Signal that we're done sending by calling finish()
            // This tells the client no more data will be sent on this stream
            send_stream.finish()?;
        }
        
        // Keep the connection alive by continuing to accept streams
        // We'll break when accept_bi fails (client closes connection)
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
