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

// No config needed - server knows its upstream, client knows its port

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
        .handle_streams(HttpProxyProtocol::Forward, move |mut session, _: ()| {
            let upstream = upstream_url.clone();
            async move {
                println!(
                    "🔀 HTTP proxy session started from {}",
                    session.peer().id52()
                );
                println!("📡 Forwarding to: {upstream}");

                // For now, just use copy_both to forward raw HTTP
                // TODO: Parse HTTP properly and forward to upstream URL
                println!("⚠️  TODO: Complete HTTP parsing and upstream forwarding");
                println!("📡 Would forward HTTP requests to: {upstream}");
                
                // For demonstration, just echo back a simple response
                let response = "HTTP/1.1 200 OK\r\nContent-Length: 25\r\n\r\nHTTP Proxy TODO Response";
                tokio::io::AsyncWriteExt::write_all(&mut session.send, response.as_bytes()).await
                    .map_err(ProxyError::Io)?;
                
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

    // Connect to P2P proxy server - no config needed
    let mut session = fastn_p2p::client::connect(
        private_key,
        target,
        HttpProxyProtocol::Forward,
        (), // No data needed
    )
    .await?;

    // Start local HTTP server that forwards via P2P
    let addr: std::net::SocketAddr = format!("127.0.0.1:{port}").parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    
    println!("🌐 Local HTTP server running on http://localhost:{port}");
    println!("🔗 All requests will be forwarded via P2P to upstream server");
    
    loop {
        match listener.accept().await {
            Ok((mut stream, _)) => {
                println!("📥 HTTP request received");
                
                // Use copy_both to bidirectionally forward HTTP traffic
                let (tcp_read, tcp_write) = stream.into_split();
                
                // This is the power of our copy_both method!
                match session.copy_both(tcp_read, tcp_write).await {
                    Ok((sent, received)) => {
                        println!("✅ HTTP request/response completed: {sent} sent, {received} received");
                    }
                    Err(e) => {
                        println!("❌ Proxy error: {e}");
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
