//! Multi-Protocol Server Example
//!
//! Demonstrates a server that handles multiple different protocols:
//! - Echo (request/response)
//! - FileDownload (streaming)
//! - Math (request/response with different types)
//!
//! Usage:
//!   multi_protocol server [key]                    # Start multi-protocol server
//!   multi_protocol echo <id52> [message]           # Send echo request
//!   multi_protocol file <id52> <filename>          # Download file
//!   multi_protocol math <id52> <operation> <a> <b> # Math operation

// Echo Protocol
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub enum EchoProtocol {
    Echo,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct EchoRequest {
    pub message: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct EchoResponse {
    pub echoed: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, thiserror::Error)]
pub enum EchoError {
    #[error("Invalid message: {0}")]
    InvalidMessage(String),
}

// File Protocol  
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub enum FileProtocol {
    Download,
}

#[derive(Debug, thiserror::Error)]
pub enum FileError {
    #[error("File not found: {0}")]
    NotFound(String),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

// Math Protocol
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub enum MathProtocol {
    Calculate,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct MathRequest {
    pub operation: String,
    pub a: f64,
    pub b: f64,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct MathResponse {
    pub result: f64,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, thiserror::Error)]
pub enum MathError {
    #[error("Unknown operation: {0}")]
    UnknownOperation(String),
    #[error("Division by zero")]
    DivisionByZero,
}

#[fastn_p2p::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    match examples::parse_cli()? {
        examples::Server {
            private_key,
            config: _,
        } => run_server(private_key).await,
        examples::Client { target, config } => {
            let command = config.first().ok_or("Command required")?.as_str();
            match command {
                "echo" => {
                    let message = config.get(1).unwrap_or(&"Hello!".to_string()).clone();
                    run_echo_client(target, message).await
                }
                "file" => {
                    let filename = config.get(1).ok_or("Filename required")?.clone();
                    run_file_client(target, filename).await
                }
                "math" => {
                    let op = config.get(1).ok_or("Operation required")?.clone();
                    let a: f64 = config.get(2).ok_or("First number required")?.parse()?;
                    let b: f64 = config.get(3).ok_or("Second number required")?.parse()?;
                    run_math_client(target, op, a, b).await
                }
                _ => {
                    eprintln!("Unknown command: {command}");
                    eprintln!("Available: echo, file, math");
                    std::process::exit(1);
                }
            }
        }
    }
}

async fn run_server(private_key: fastn_p2p::SecretKey) -> Result<(), Box<dyn std::error::Error>> {
    println!("🎧 Multi-protocol server listening on: {}", private_key.id52());
    println!("📋 Supported protocols: Echo, FileDownload, Math");

    // Multi-protocol server - chain multiple handlers!
    fastn_p2p::listen(private_key)
        .handle_requests(EchoProtocol::Echo, echo_handler)
        .handle_streams(FileProtocol::Download, (), file_handler)
        .handle_requests(MathProtocol::Calculate, math_handler)
        .await?;

    Ok(())
}

async fn run_echo_client(
    target: fastn_p2p::PublicKey,
    message: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let private_key = fastn_p2p::SecretKey::generate();
    println!("📤 Sending echo: '{message}' to {target}");

    let request = EchoRequest { message };
    let result: Result<EchoResponse, EchoError> =
        fastn_p2p::client::call(private_key, target, EchoProtocol::Echo, request).await?;

    match result {
        Ok(response) => println!("✅ Echo response: {}", response.echoed),
        Err(error) => println!("❌ Echo error: {error:?}"),
    }
    Ok(())
}

async fn run_file_client(
    target: fastn_p2p::PublicKey,
    filename: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let private_key = fastn_p2p::SecretKey::generate();
    println!("📥 Requesting file '{filename}' from {target}");

    let mut session = fastn_p2p::client::connect(
        private_key,
        target,
        FileProtocol::Download,
        filename.clone(),
    )
    .await?;

    let local_filename = format!("downloaded_{filename}");
    let mut output_file = tokio::fs::File::create(&local_filename).await?;
    let bytes_copied = session.copy_to(&mut output_file).await?;

    println!("✅ Downloaded {filename} ({bytes_copied} bytes)");
    println!("💾 Saved as: {local_filename}");
    Ok(())
}

async fn run_math_client(
    target: fastn_p2p::PublicKey,
    operation: String,
    a: f64,
    b: f64,
) -> Result<(), Box<dyn std::error::Error>> {
    let private_key = fastn_p2p::SecretKey::generate();
    println!("🔢 Calculating: {a} {operation} {b}");

    let request = MathRequest { operation, a, b };
    let result: Result<MathResponse, MathError> =
        fastn_p2p::client::call(private_key, target, MathProtocol::Calculate, request).await?;

    match result {
        Ok(response) => println!("✅ Result: {}", response.result),
        Err(error) => println!("❌ Math error: {error:?}"),
    }
    Ok(())
}

// Echo handler (request/response)
async fn echo_handler(req: EchoRequest) -> Result<EchoResponse, EchoError> {
    println!("💬 Echo request: {}", req.message);
    Ok(EchoResponse {
        echoed: format!("Echo: {}", req.message),
    })
}

// File handler (streaming)
async fn file_handler(
    mut session: fastn_p2p::Session<FileProtocol>,
    filename: String,
    _state: (),
) -> Result<(), FileError> {
    println!("📂 File request: '{filename}' from {}", session.peer().id52());

    if filename.contains("..") || filename.contains('/') {
        return Err(FileError::PermissionDenied(filename));
    }

    let mut file = tokio::fs::File::open(&filename)
        .await
        .map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => FileError::NotFound(filename.clone()),
            std::io::ErrorKind::PermissionDenied => FileError::PermissionDenied(filename.clone()),
            _ => FileError::Io(e),
        })?;

    let bytes_sent = session.copy_from(&mut file).await.map_err(FileError::Io)?;
    println!("✅ Sent {filename} ({bytes_sent} bytes)");
    Ok(())
}

// Math handler (request/response)
async fn math_handler(req: MathRequest) -> Result<MathResponse, MathError> {
    println!("🔢 Math request: {} {} {}", req.a, req.operation, req.b);

    let result = match req.operation.as_str() {
        "+" => req.a + req.b,
        "-" => req.a - req.b,
        "*" => req.a * req.b,
        "/" => {
            if req.b == 0.0 {
                return Err(MathError::DivisionByZero);
            }
            req.a / req.b
        }
        _ => return Err(MathError::UnknownOperation(req.operation)),
    };

    Ok(MathResponse { result })
}