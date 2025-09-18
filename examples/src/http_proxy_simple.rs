//! Simple HTTP Proxy Demo
//!
//! Basic demonstration of HTTP forwarding pattern.
//! For production HTTP proxy, see malai-next/expose_http and malai-next/http_bridge.
//!
//! Usage:
//!   http_proxy_simple server [upstream]   # Start proxy server  
//!   http_proxy_simple client <id52>       # Connect to proxy

// Protocol definition
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub enum HttpProtocol {
    Forward,
}

// Simple HTTP request structure
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct HttpRequest {
    pub method: String,
    pub path: String,
}

// Custom error type
#[derive(Debug, thiserror::Error)]
pub enum ProxyError {
    #[error("HTTP error: {0}")]
    Http(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[fastn_p2p::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    match examples::parse_cli()? {
        examples::Server { private_key, config } => {
            let upstream = config.first().unwrap_or(&"http://httpbin.org".to_string()).clone();
            run_server(private_key, upstream).await
        }
        examples::Client { target, config: _ } => {
            run_client(target).await
        }
    }
}

async fn run_server(
    private_key: fastn_p2p::SecretKey,
    upstream_url: String,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔀 Simple HTTP proxy server listening on: {}", private_key.id52());
    println!("📡 Demo forwarding to: {upstream_url}");
    println!("⚠️  For production use: see malai-next/expose_http");

    fastn_p2p::listen(private_key)
        .handle_streams(HttpProtocol::Forward, upstream_url, demo_proxy_handler)
        .await?;

    Ok(())
}

async fn run_client(target: fastn_p2p::PublicKey) -> Result<(), Box<dyn std::error::Error>> {
    let private_key = fastn_p2p::SecretKey::generate();

    println!("📤 Sending demo HTTP request to {target}");
    println!("⚠️  For production use: see malai-next/http_bridge");

    let request = HttpRequest {
        method: "GET".to_string(),
        path: "/ip".to_string(),
    };

    let mut session = fastn_p2p::client::connect(
        private_key,
        target,
        HttpProtocol::Forward,
        request,
    ).await?;

    // Read demo response using copy_to method
    let mut response_writer = Vec::new();
    session.copy_to(&mut response_writer).await?;
    
    println!("✅ Response: {}", String::from_utf8_lossy(&response_writer));
    Ok(())
}

// Demo handler - see malai-next for production implementation
async fn demo_proxy_handler(
    mut session: fastn_p2p::Session<HttpProtocol>,
    http_request: HttpRequest,
    upstream_url: String,
) -> Result<(), ProxyError> {
    println!(
        "🔀 Demo HTTP request: {} {} from {}",
        http_request.method, http_request.path,
        session.peer().id52()
    );
    
    println!("📡 Would forward to: {upstream_url}{}", http_request.path);
    
    // Demo response
    let response = "HTTP/1.1 200 OK\r\nContent-Length: 30\r\n\r\nDemo HTTP Proxy Response - OK!";
    tokio::io::AsyncWriteExt::write_all(&mut session.send, response.as_bytes()).await?;
    
    Ok(())
}