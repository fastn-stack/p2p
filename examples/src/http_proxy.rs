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
pub enum HttpProxyProtocol { Forward }

// Configuration sent during connection setup
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct ProxyConfig {
    pub upstream_url: String,  // Where server forwards requests
    pub local_port: u16,       // Where client listens
}

#[fastn_context::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    match examples::parse_cli()? {
        examples::Server { private_key, config } => {
            let upstream = config.first().unwrap_or(&"http://httpbin.org".to_string()).clone();
            run_server(private_key, upstream).await
        }
        examples::Client { target, config } => {
            let port: u16 = config.first().unwrap_or(&"8080".to_string()).parse().unwrap_or(8080);
            run_client(target, port).await
        }
    }
}

async fn run_server(private_key: fastn_id52::SecretKey, upstream_url: String) -> Result<(), Box<dyn std::error::Error>> {
    
    println!("🔀 HTTP proxy server listening on: {}", private_key.id52());
    println!("📡 Forwarding to upstream: {}", upstream_url);
    
    // Store upstream URL for the handler to use
    let config = ProxyConfig { 
        upstream_url: upstream_url.clone(),
        local_port: 0, // Not used on server side
    };
    
    fastn_p2p::listen(private_key)
        .handle_streams(HttpProxyProtocol::Forward, move |session, _config: ProxyConfig| {
            let upstream = upstream_url.clone();
            async move { http_proxy_handler(session, upstream).await }
        })
        .await?;
        
    Ok(())
}

async fn run_client(target: fastn_id52::PublicKey, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let private_key = fastn_id52::SecretKey::generate();

    println!("🌐 Starting local HTTP server on port {}", port);
    println!("🔗 Forwarding to P2P server: {}", target);

    // Connect to P2P proxy server
    let config = ProxyConfig {
        upstream_url: "unused".to_string(), // Server will use its own upstream
        local_port: port,
    };
    
    let mut session = fastn_p2p::client::connect(
        private_key,
        target,
        HttpProxyProtocol::Forward,
        config,
    ).await?;

    // TODO: Start local HTTP server that forwards requests via session
    println!("⚠️  TODO: Implement local HTTP server that forwards via P2P session");
    
    // For now, just keep the connection alive
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}

// HTTP proxy handler - forwards requests to upstream server
async fn http_proxy_handler(mut session: fastn_p2p::Session<HttpProxyProtocol>, upstream_url: String) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("🔀 HTTP proxy session started from {}", session.peer().id52());
    println!("📡 Will forward to: {}", upstream_url);
    
    // TODO: Parse HTTP requests from session.recv
    // TODO: Forward to upstream_url using reqwest or hyper
    // TODO: Stream response back via session.send
    
    println!("⚠️  TODO: Implement HTTP request forwarding");
    Ok(())
}