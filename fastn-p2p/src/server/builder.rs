/// Server builder for clean multi-protocol server setup
///
/// Also implements Future so you can .await on it to start the server
pub struct ServerBuilder {
    private_key: fastn_id52::SecretKey,
    request_handlers: std::collections::HashMap<serde_json::Value, RequestHandler>,
    stream_handlers: std::collections::HashMap<serde_json::Value, StreamHandler>,
    connection_auth: Option<ConnectionAuthHook>,
    stream_auth: Option<StreamAuthHook>,
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

/// Connection authorization hook - called when a peer connects
/// Returns true to allow connection, false to deny
type ConnectionAuthHook = Box<
    dyn Fn(
        &fastn_id52::PublicKey,  // peer connecting
    ) -> bool
        + Send
        + Sync,
>;

/// Stream authorization hook - called when a peer opens a stream
/// Returns true to allow stream, false to deny
type StreamAuthHook = Box<
    dyn Fn(
        &fastn_id52::PublicKey,  // peer making the request
        &serde_json::Value,      // protocol being requested
        &serde_json::Value,      // initial data sent with request
    ) -> bool
        + Send
        + Sync,
>;

impl ServerBuilder {
    pub fn new(private_key: fastn_id52::SecretKey) -> Self {
        Self {
            private_key,
            request_handlers: std::collections::HashMap::new(),
            stream_handlers: std::collections::HashMap::new(),
            connection_auth: None,
            stream_auth: None,
            server_task: None,
        }
    }

    /// Set connection authorization hook - called when any peer connects
    /// 
    /// # Example
    /// ```rust
    /// fastn_p2p::listen(key)
    ///     .with_connection_auth(|peer| {
    ///         // Only allow connections from known peers
    ///         ALLOWED_PEERS.contains(peer)
    ///     })
    ///     .handle_requests(Protocol::Echo, echo_handler)
    ///     .await?;
    /// ```
    pub fn with_connection_auth<F>(mut self, auth_fn: F) -> Self
    where
        F: Fn(&fastn_id52::PublicKey) -> bool + Send + Sync + 'static,
    {
        self.connection_auth = Some(Box::new(auth_fn));
        self
    }
    
    /// Set stream authorization hook - called when a peer opens a stream
    /// 
    /// # Example
    /// ```rust
    /// fastn_p2p::listen(key)
    ///     .with_stream_auth(|peer, protocol, data| {
    ///         // Allow different access based on protocol
    ///         match protocol {
    ///             p if p == &json!("Admin") => ADMIN_PEERS.contains(peer),
    ///             _ => true
    ///         }
    ///     })
    ///     .handle_requests(Protocol::Echo, echo_handler)
    ///     .await?;
    /// ```
    pub fn with_stream_auth<F>(mut self, auth_fn: F) -> Self
    where
        F: Fn(&fastn_id52::PublicKey, &serde_json::Value, &serde_json::Value) -> bool
            + Send
            + Sync
            + 'static,
    {
        self.stream_auth = Some(Box::new(auth_fn));
        self
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
            let connection_auth = self.connection_auth.take();
            let stream_auth = self.stream_auth.take();
            
            println!("🎧 Server listening on: {}", private_key.id52());
            
            // Create the server future
            self.server_task = Some(Box::pin(run_server(
                private_key, 
                request_handlers, 
                stream_handlers, 
                connection_auth,
                stream_auth
            )));
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
    connection_auth: Option<ConnectionAuthHook>,
    stream_auth: Option<StreamAuthHook>,
) -> Result<(), Box<dyn std::error::Error>> {
    let server_public_key = private_key.public_key();
    // Get endpoint for listening
    let endpoint = fastn_net::get_endpoint(private_key).await?;
    
    // Wrap handlers in Arc for sharing across tasks
    let request_handlers = std::sync::Arc::new(request_handlers);
    let stream_handlers = std::sync::Arc::new(stream_handlers);
    let connection_auth = connection_auth.map(std::sync::Arc::new);
    let stream_auth = stream_auth.map(std::sync::Arc::new);
    
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
                let connection_auth = connection_auth.clone();
                let stream_auth = stream_auth.clone();
                let server_key = server_public_key.clone();
                crate::spawn(async move {
                    if let Err(e) = handle_connection(
                        conn, 
                        server_key,
                        &request_handlers, 
                        &stream_handlers, 
                        connection_auth.as_deref(),
                        stream_auth.as_deref()
                    ).await {
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
    server_key: fastn_id52::PublicKey,
    request_handlers: &std::collections::HashMap<serde_json::Value, RequestHandler>,
    stream_handlers: &std::collections::HashMap<serde_json::Value, StreamHandler>,
    connection_auth: Option<&ConnectionAuthHook>,
    stream_auth: Option<&StreamAuthHook>,
) -> Result<(), Box<dyn std::error::Error>> {
    let conn = conn.await?;
    
    // Get peer's ID52 for logging and security
    let peer_key = fastn_net::get_remote_id52(&conn).await?;
    tracing::debug!("Connection established with peer: {}", peer_key.id52());
    
    // HANDSHAKE: Wait for the first stream which MUST be the handshake
    let (protocol, mut send_stream, mut recv_stream) = 
        fastn_net::accept_bi(&conn, &[fastn_net::Protocol::Generic(
            serde_json::Value::String(crate::handshake::HANDSHAKE_PROTOCOL.to_string())
        )]).await?;
    
    // Verify it's the handshake protocol
    match protocol {
        fastn_net::Protocol::Generic(json)
            if json == serde_json::Value::String(crate::handshake::HANDSHAKE_PROTOCOL.to_string()) => {
            // Good, this is handshake
        }
        other => {
            tracing::warn!("First stream was not handshake: {:?}", other);
            conn.close(0u8.into(), b"Handshake required");
            return Ok(());
        }
    };
    
    // Read ClientHello
    let client_hello: crate::handshake::ClientHello = match fastn_net::next_json(&mut recv_stream).await {
        Ok(hello) => hello,
        Err(e) => {
            tracing::warn!("Failed to read ClientHello: {}", e);
            conn.close(0u8.into(), b"Invalid handshake");
            return Ok(());
        }
    };
    
    tracing::debug!("Received ClientHello from {} ({}): {} protocols supported", 
                   client_hello.client_name, client_hello.client_version, 
                   client_hello.supported_protocols.len());
    
    // Check connection-level authorization with client info
    if let Some(auth) = connection_auth {
        if !auth(&peer_key) {
            tracing::warn!("Connection denied for peer {}", peer_key.id52());
            let response = crate::handshake::ServerHello::failure(
                crate::handshake::HandshakeError::Unauthorized
            );
            let json = serde_json::to_string(&response)?;
            send_stream.write_all(json.as_bytes()).await?;
            send_stream.write_all(b"\n").await?;
            send_stream.finish()?;
            conn.close(0u8.into(), b"Unauthorized");
            return Ok(());
        }
    }
    
    // Filter protocols - only include ones we actually support
    let mut accepted_protocols = Vec::new();
    for protocol in &client_hello.supported_protocols {
        if request_handlers.contains_key(protocol) || stream_handlers.contains_key(protocol) {
            accepted_protocols.push(protocol.clone());
        }
    }
    
    // Send ServerHello
    let server_hello = if !accepted_protocols.is_empty() {
        let mut hello = crate::handshake::ServerHello::success();
        if let crate::handshake::ServerHello::Success { accepted_protocols: ref mut protocols, .. } = hello {
            *protocols = accepted_protocols;
        }
        hello
    } else {
        crate::handshake::ServerHello::failure(
            crate::handshake::HandshakeError::NoCommonProtocols
        )
    };
    
    let json = serde_json::to_string(&server_hello)?;
    send_stream.write_all(json.as_bytes()).await?;
    send_stream.write_all(b"\n").await?;
    send_stream.finish()?;
    
    if matches!(server_hello, crate::handshake::ServerHello::Failure { .. }) {
        conn.close(0u8.into(), b"No compatible protocols");
        return Ok(());
    }
    
    let protocol_count = if let crate::handshake::ServerHello::Success { ref accepted_protocols, .. } = server_hello {
        accepted_protocols.len()
    } else {
        0
    };
    tracing::info!("Handshake complete with {} - {} protocols enabled", 
                  client_hello.client_name, protocol_count);
    
    // Now we can accept application protocol streams
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
        
        // Check stream-level authorization if hook is provided
        if let Some(auth) = stream_auth {
            if !auth(&peer_key, &wrapper.protocol, &wrapper.data) {
                tracing::warn!("Stream authorization denied for peer {} protocol {:?}", 
                            peer_key.id52(), wrapper.protocol);
                let error_msg = "Authorization denied";
                send_stream.write_all(error_msg.as_bytes()).await?;
                send_stream.write_all(b"\n").await?;
                send_stream.finish()?;
                continue;
            }
        }
        
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
