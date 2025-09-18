//! HTTP Proxy Example
//!
//! Client runs local HTTP server, forwards all requests to P2P server.
//! Server forwards requests to configured upstream HTTP server.
//!
//! NOTE: This is a simplified example for demonstration. For production
//! HTTP proxying, see fastn-net's http_to_peer/peer_to_http which uses
//! hyper for proper HTTP/1.1 support with connection reuse.
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
                
                // Connect to upstream server
                let upstream_host = upstream.trim_start_matches("http://").trim_start_matches("https://");
                let upstream_port = if upstream.starts_with("https://") { 443 } else { 80 };
                let upstream_addr = format!("{}:{}", upstream_host, upstream_port);
                
                println!("📡 Connecting to upstream: {upstream_addr}");
                
                match tokio::net::TcpStream::connect(&upstream_addr).await {
                    Ok(upstream_stream) => {
                        println!("✅ Connected to upstream server");
                        
                        // Reconstruct HTTP request for upstream
                        let mut request_lines = vec![
                            format!("{} {} {}", http_request.method, http_request.path, http_request.version)
                        ];
                        
                        // Add headers
                        for (key, value) in http_request.headers {
                            request_lines.push(format!("{}: {}", key, value));
                        }
                        request_lines.push(String::new()); // Empty line before body
                        
                        let request_header = request_lines.join("\r\n");
                        
                        // Send HTTP request header to upstream
                        let (mut upstream_read, mut upstream_write) = upstream_stream.into_split();
                        tokio::io::AsyncWriteExt::write_all(&mut upstream_write, request_header.as_bytes()).await
                            .map_err(ProxyError::Io)?;
                        
                        // Now use copy_both for efficient bidirectional body streaming!
                        match session.copy_both(upstream_read, upstream_write).await {
                            Ok((from_upstream, to_upstream)) => {
                                println!("✅ HTTP proxy completed: {to_upstream} sent to upstream, {from_upstream} received");
                            }
                            Err(e) => {
                                println!("❌ Proxy stream error: {e}");
                                return Err(ProxyError::Io(e));
                            }
                        }
                    }
                    Err(e) => {
                        println!("❌ Failed to connect to upstream: {e}");
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
                
                // Connect with HTTP request headers as data
                let mut session = fastn_p2p::client::connect(
                    private_key.clone(),
                    target,
                    HttpProxyProtocol::Forward,
                    http_request,
                ).await?;
                
                // Use copy_both for bidirectional HTTP body streaming!
                let (tcp_read, tcp_write) = tcp_stream.into_split();
                match session.copy_both(tcp_read, tcp_write).await {
                    Ok((to_p2p, from_p2p)) => {
                        println!("✅ HTTP proxy completed: {to_p2p} sent via P2P, {from_p2p} received");
                    }
                    Err(e) => {
                        println!("❌ Client proxy error: {e}");
                    }
                }
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
