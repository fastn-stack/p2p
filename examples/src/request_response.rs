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

#[derive(clap::Parser)]
struct Args {
    #[command(subcommand)]
    mode: Mode,
}

#[derive(clap::Subcommand)]
enum Mode {
    /// Start server (listens for requests)
    Server {
        /// Optional private key
        #[arg(long)]
        key: Option<String>,
    },
    /// Send request to server  
    Client {
        /// Target ID52
        target: String,
        /// Message to send
        #[arg(default_value = "Hello P2P!")]
        message: String,
    },
}

#[fastn_context::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = <Args as clap::Parser>::parse();

    match args.mode {
        Mode::Server { key } => run_server(key).await,
        Mode::Client { target, message } => run_client(target, message).await,
    }
}

async fn run_server(key: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let private_key = examples::key_from_str_or_generate(key.as_deref())?;
    
    println!("🎧 Server listening on: {}", private_key.id52());
    
    fastn_p2p::listen(private_key)
        .handle_requests(EchoProtocol::Echo, echo_handler)
        .await?;
        
    Ok(())
}

async fn run_client(target: String, message: String) -> Result<(), Box<dyn std::error::Error>> {
    let private_key = fastn_id52::SecretKey::generate();
    let target_key = examples::parse_peer_id(&target)?;

    println!("📤 Sending '{}' to {}", message, target);

    let request = EchoRequest { message };
    let result: EchoResult = fastn_p2p::client::call(
        private_key,
        target_key,
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
