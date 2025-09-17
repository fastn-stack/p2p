//! Request/Response Pattern Example
//! 
//! Showcases the classic client-server request/response pattern over P2P.
//! Send a request and get a response back - like HTTP but over P2P.
//! 
//! Usage: 
//!   request_response receiver [key]     # Start server mode  
//!   request_response sender <id52> [msg] # Send request to server

use clap::{Parser, Subcommand};
use fastn_p2p::client::call;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};

// Protocol Definition - shared between sender and receiver
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum EchoProtocol { Echo }

impl std::fmt::Display for EchoProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "echo")
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EchoRequest { pub message: String }

#[derive(Serialize, Deserialize, Debug)]  
pub struct EchoResponse { pub echoed: String }

#[derive(Serialize, Deserialize, Debug)]
pub enum EchoError { InvalidMessage(String) }

type EchoResult = Result<EchoResponse, EchoError>;

#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    mode: Mode,
}

#[derive(Subcommand)]
enum Mode {
    /// Start receiver (listens for messages)
    Receiver {
        /// Optional private key
        #[arg(long)]
        key: Option<String>,
    },
    /// Send message to receiver  
    Sender {
        /// Target ID52
        target: String,
        /// Message to send
        #[arg(default_value = "Hello P2P!")]
        message: String,
    },
}

#[fastn_context::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    match args.mode {
        Mode::Receiver { key } => run_receiver(key).await,
        Mode::Sender { target, message } => run_sender(target, message).await,
    }
}

async fn run_receiver(key: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let private_key = match key {
        Some(k) => k.parse()?,
        None => fastn_id52::SecretKey::generate(),
    };

    println!("🎧 Listening on: {}", private_key.id52());
    
    let protocols = [EchoProtocol::Echo];
    let mut stream = fastn_p2p::listen!(private_key, &protocols);

    while let Some(request) = stream.next().await {
        let request = request?;
        println!("📨 Message from {}", request.peer().id52());
        
        request.handle(|req: EchoRequest| async move {
            println!("💬 Received: {}", req.message);
            Ok::<EchoResponse, EchoError>(EchoResponse { echoed: format!("Echo: {}", req.message) })
        }).await?;
    }
    Ok(())
}

async fn run_sender(target: String, message: String) -> Result<(), Box<dyn std::error::Error>> {
    let private_key = fastn_id52::SecretKey::generate();
    let target_key: fastn_id52::PublicKey = target.parse()?;

    println!("📤 Sending '{}' to {}", message, target);

    let request = EchoRequest { message };
    let result: EchoResult = call(
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