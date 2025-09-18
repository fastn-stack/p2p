//! Request/Response Pattern Example
//! 
//! Showcases the classic client-server request/response pattern over P2P.
//! Send a request and get a response back - like HTTP but over P2P.
//! 
//! Usage: 
//!   request_response server [key]       # Start server mode  
//!   request_response client <id52> [msg] # Send request to server

// Protocol Definition - shared between client and server
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub enum EchoProtocol { Echo }

// No Display implementation needed - Debug and Serialize are sufficient!

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct EchoRequest { pub message: String }

#[derive(serde::Serialize, serde::Deserialize, Debug)]  
pub struct EchoResponse { pub echoed: String }

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum EchoError { InvalidMessage(String) }

type EchoResult = Result<EchoResponse, EchoError>;

#[fastn_context::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = examples::parse_cli()?;

    match args.mode {
        examples::Server { private_key, config: _ } => run_server(private_key).await,
        examples::Client { target, config } => {
            let message = config.first().unwrap_or(&"Hello P2P!".to_string()).clone();
            run_client(target, message).await
        }
    }
}

async fn run_server(private_key: fastn_id52::SecretKey) -> Result<(), Box<dyn std::error::Error>> {
    println!("🎧 Server listening on: {}", private_key.id52());
    
    fastn_p2p::listen(private_key)
        .handle_requests(EchoProtocol::Echo, echo_handler)
        .await?;
        
    Ok(())
}

async fn run_client(target: fastn_id52::PublicKey, message: String) -> Result<(), Box<dyn std::error::Error>> {
    let private_key = fastn_id52::SecretKey::generate();

    println!("📤 Sending '{}' to {}", message, target);

    let request = EchoRequest { message };
    let result: EchoResult = fastn_p2p::client::call(
        private_key,
        target,
        EchoProtocol::Echo,
        request,
    ).await?;

    match result {
        Ok(response) => println!("✅ Response: {}", response.echoed),
        Err(error) => println!("❌ Error: {:?}", error),
    }
    Ok(())
}

// Request handler function - clean and simple!
async fn echo_handler(req: EchoRequest) -> Result<EchoResponse, EchoError> {
    println!("💬 Received: {}", req.message);
    Ok(EchoResponse { echoed: format!("Echo: {}", req.message) })
}
