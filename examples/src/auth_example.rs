//! Authorization Hook Example
//!
//! Demonstrates how to use authorization hooks to control access to protocols
//! based on peer identity and request data.
//!
//! Usage:
//!   auth_example server [key]          # Start server with auth
//!   auth_example client <id52> admin   # Try admin protocol (will be denied)
//!   auth_example client <id52> echo    # Try echo protocol (allowed)

// Protocol Definitions
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub enum Protocol {
    Echo,
    Admin,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Request {
    pub message: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct Response {
    pub result: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, thiserror::Error)]
pub enum AppError {
    #[error("Unauthorized")]
    Unauthorized,
}

#[fastn_p2p::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    match examples::parse_cli()? {
        examples::Server {
            private_key,
            config: _,
        } => run_server(private_key).await,
        examples::Client { target, config } => {
            let protocol = match config.first().map(String::as_str) {
                Some("admin") => Protocol::Admin,
                _ => Protocol::Echo,
            };
            let message = config.get(1).unwrap_or(&"Test message".to_string()).clone();
            run_client(target, protocol, message).await
        }
    }
}

async fn run_server(private_key: fastn_p2p::SecretKey) -> Result<(), Box<dyn std::error::Error>> {
    // Define allowed admin peers (in production, load from config)
    let admin_peers: Vec<String> = vec![
        // Add specific peer IDs that should have admin access
        // "adminpeer123...".to_string(),
    ];
    
    println!("🎧 Server listening on: {}", private_key.id52());
    println!("🔒 Authorization enabled:");
    println!("   - Echo protocol: Open to all");
    println!("   - Admin protocol: Restricted to {} peers", admin_peers.len());

    fastn_p2p::listen(private_key)
        .with_connection_auth(move |peer| {
            // Could implement IP-based blocking, rate limiting, etc.
            println!("🔗 Connection from peer: {}", peer.id52());
            true // Allow all connections for this example
        })
        .with_stream_auth(move |peer, protocol, data| {
            // Convert protocol to our enum for checking
            let protocol_str = protocol.as_str().unwrap_or("");
            
            println!("🔑 Stream auth check: peer={} protocol={} data={}", 
                    peer.id52(), protocol_str, data);
            
            match protocol_str {
                "\"Echo\"" => {
                    println!("✅ Allowing Echo protocol for peer {}", peer.id52());
                    true // Echo is open to everyone
                }
                "\"Admin\"" => {
                    let allowed = admin_peers.contains(&peer.id52());
                    if allowed {
                        println!("✅ Allowing Admin protocol for authorized peer {}", peer.id52());
                    } else {
                        println!("❌ Denying Admin protocol for unauthorized peer {}", peer.id52());
                    }
                    allowed
                }
                _ => {
                    println!("❓ Unknown protocol: {}", protocol_str);
                    false // Deny unknown protocols
                }
            }
        })
        .handle_requests(Protocol::Echo, echo_handler)
        .handle_requests(Protocol::Admin, admin_handler)
        .await?;

    Ok(())
}

async fn run_client(
    target: fastn_p2p::PublicKey,
    protocol: Protocol,
    message: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let private_key = fastn_p2p::SecretKey::generate();

    println!("📤 Attempting {} protocol with message: {}", 
            match protocol {
                Protocol::Echo => "Echo",
                Protocol::Admin => "Admin",
            }, 
            message);
    println!("🔑 Our ID: {}", private_key.id52());

    let request = Request { message };
    let result: Result<Response, AppError> =
        fastn_p2p::client::call(private_key, target, protocol, request).await?;

    match result {
        Ok(response) => println!("✅ Success: {}", response.result),
        Err(error) => println!("❌ Error: {:?}", error),
    }
    Ok(())
}

async fn echo_handler(req: Request) -> Result<Response, AppError> {
    println!("💬 Echo handler: {}", req.message);
    Ok(Response {
        result: format!("Echo: {}", req.message),
    })
}

async fn admin_handler(req: Request) -> Result<Response, AppError> {
    println!("🛡️ Admin handler: {}", req.message);
    Ok(Response {
        result: format!("Admin command executed: {}", req.message),
    })
}