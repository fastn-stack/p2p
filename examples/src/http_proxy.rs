//! HTTP Proxy Example
//!
//! Client runs local HTTP server, forwards all requests to P2P server.
//! Server forwards requests to configured upstream HTTP server.
//!
//! Usage:
//!   http_proxy server [key] [upstream_url]     # Start proxy server  
//!   http_proxy client <id52> [local_port]      # Start local HTTP server

// Protocol Definition
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub enum HttpProxyProtocol {
    Forward,
}

// HTTP request representation (headers only)
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct HttpRequest {
    pub method: String,
    pub path: String,
    pub headers: std::collections::HashMap<String, String>,
    pub version: String,
}

#[fastn_p2p::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    match examples::parse_cli()? {
        examples::Server {
            private_key,
            config,
        } => {
            let upstream = config
                .first()
                .unwrap_or(&"http://httpbin.org".to_string())
                .clone();
            run_server(private_key, upstream).await
        }
        examples::Client { target, config } => {
            let port: u16 = config
                .first()
                .unwrap_or(&"8080".to_string())
                .parse()
                .unwrap_or(8080);
            run_client(target, port).await
        }
    }
}

async fn run_server(
    private_key: fastn_p2p::SecretKey,
    upstream_url: String,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔀 HTTP proxy server listening on: {}", private_key.id52());
    println!("📡 Forwarding to upstream: {}", upstream_url);

    fastn_p2p::listen(private_key)
        .handle_streams(HttpProxyProtocol::Forward, move |mut session, http_request: HttpRequest| {
            let upstream = upstream_url.clone();
            async move {
                println!(
                    "🔀 HTTP proxy session for {} {} from {}",
                    http_request.method, http_request.path,
                    session.peer().id52()
                );
                
                // Construct upstream URL
                let full_url = format!("{upstream}{}", http_request.path);
                println!("📡 Forwarding to: {full_url}");
                
                // Create HTTP client and forward request
                let client = reqwest::Client::new();
                let mut request_builder = match http_request.method.as_str() {
                    "GET" => client.get(&full_url),
                    "POST" => client.post(&full_url),
                    "PUT" => client.put(&full_url),
                    "DELETE" => client.delete(&full_url),
                    _ => {
                        println!("⚠️ Unsupported HTTP method: {}", http_request.method);
                        return Ok(());
                    }
                };
                
                // Add headers from original request
                for (key, value) in http_request.headers {
                    request_builder = request_builder.header(&key, &value);
                }
                
                // Send request to upstream
                match request_builder.send().await {
                    Ok(response) => {
                        let status = response.status();
                        let headers = response.headers().clone();
                        let body = response.bytes().await.map_err(|e| ProxyError::Http(e.to_string()))?;
                        
                        // Build HTTP response
                        let mut response_lines = vec![
                            format!("HTTP/1.1 {} {}", status.as_u16(), status.canonical_reason().unwrap_or("OK"))
                        ];
                        
                        // Add response headers
                        for (name, value) in headers.iter() {
                            response_lines.push(format!("{}: {}", name, value.to_str().unwrap_or("")));
                        }
                        
                        response_lines.push(String::new()); // Empty line before body
                        let response_header = response_lines.join("\r\n");
                        
                        // Send response over P2P
                        tokio::io::AsyncWriteExt::write_all(&mut session.send, response_header.as_bytes()).await
                            .map_err(ProxyError::Io)?;
                        tokio::io::AsyncWriteExt::write_all(&mut session.send, &body).await
                            .map_err(ProxyError::Io)?;
                        
                        println!("✅ Forwarded response: {} ({} bytes)", status, body.len());
                    }
                    Err(e) => {
                        println!("❌ Upstream error: {e}");
                        let error_response = "HTTP/1.1 502 Bad Gateway\r\nContent-Length: 19\r\n\r\nUpstream Unavailable";
                        tokio::io::AsyncWriteExt::write_all(&mut session.send, error_response.as_bytes()).await
                            .map_err(ProxyError::Io)?;
                    }
                }
                
                Ok::<(), ProxyError>(())
            }
        })
        .await?;

    Ok(())
}

async fn run_client(
    target: fastn_p2p::PublicKey,
    port: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    let private_key = fastn_p2p::SecretKey::generate();

    println!("🌐 Starting local HTTP server on port {}", port);
    println!("🔗 Forwarding to P2P server: {}", target);

    // Start local HTTP server that forwards via P2P
    let addr: std::net::SocketAddr = format!("127.0.0.1:{port}").parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    
    println!("🌐 Local HTTP server running on http://localhost:{port}");
    println!("🔗 All requests will be forwarded via P2P to upstream server");
    
    loop {
        match listener.accept().await {
            Ok((mut tcp_stream, _)) => {
                println!("📥 HTTP request received");
                
                // Parse HTTP request from TCP stream
                let mut request_buffer = Vec::new();
                tokio::io::AsyncReadExt::read_to_end(&mut tcp_stream, &mut request_buffer).await?;
                
                if request_buffer.is_empty() {
                    continue;
                }
                
                // Parse HTTP request (simplified header parsing)
                let request_str = String::from_utf8_lossy(&request_buffer);
                let lines: Vec<&str> = request_str.lines().collect();
                
                if lines.is_empty() {
                    continue;
                }
                
                // Parse request line
                let request_line = lines[0];
                let parts: Vec<&str> = request_line.split_whitespace().collect();
                if parts.len() < 3 {
                    continue;
                }
                
                let method = parts[0].to_string();
                let path = parts[1].to_string();
                let version = parts[2].to_string();
                
                // Parse headers
                let mut headers = std::collections::HashMap::new();
                for line in &lines[1..] {
                    if line.is_empty() {
                        break; // End of headers
                    }
                    if let Some((key, value)) = line.split_once(": ") {
                        headers.insert(key.to_string(), value.to_string());
                    }
                }
                
                let http_request = HttpRequest {
                    method: method.clone(),
                    path: path.clone(),
                    headers,
                    version,
                };
                
                println!("📤 Forwarding {method} {path} via P2P");
                
                // Connect with HTTP request as data
                let mut session = fastn_p2p::client::connect(
                    private_key.clone(),
                    target,
                    HttpProxyProtocol::Forward,
                    http_request,
                ).await?;
                
                // Stream response back to HTTP client
                session.copy_to(&mut tcp_stream).await?;
                println!("✅ HTTP response returned to client");
            }
            Err(e) => {
                println!("❌ Failed to accept connection: {e}");
            }
        }
    }
}

// Custom error type for HTTP proxy
#[derive(Debug, thiserror::Error)]
pub enum ProxyError {
    #[error("HTTP error: {0}")]
    Http(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

// Note: Handler logic moved inline to capture upstream_url from closure
