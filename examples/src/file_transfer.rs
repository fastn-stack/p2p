//! File Transfer Example
//! 
//! Stream files directly over P2P without loading into memory.
//! Uses async I/O copy for efficient streaming.
//! 
//! Usage: 
//!   file_transfer server [key]              # Start file server
//!   file_transfer client <id52> <filename>  # Download file

// Protocol Definition
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub enum FileProtocol { Download }

#[fastn_context::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = examples::parse_cli()?;

    match args.mode {
        examples::Server { private_key, config: _ } => run_server(private_key).await,
        examples::Client { target, config } => {
            let filename = config.first().ok_or("Filename required")?.clone();
            run_client(target, filename).await
        }
    }
}

async fn run_server(private_key: fastn_id52::SecretKey) -> Result<(), Box<dyn std::error::Error>> {
    
    println!("📁 File server listening on: {}", private_key.id52());
    println!("⚠️  Security: Only files in current directory are served!");
    
    fastn_p2p::listen(private_key)
        .handle_streams(FileProtocol::Download, file_stream_handler)
        .await?;
        
    Ok(())
}

async fn run_client(target: fastn_id52::PublicKey, filename: String) -> Result<(), Box<dyn std::error::Error>> {
    let private_key = fastn_id52::SecretKey::generate();

    println!("📥 Requesting file '{}' from {}", filename, target);

    // Connect with protocol + filename data - no manual stream writing needed!
    let mut session = fastn_p2p::client::connect(
        private_key,
        target,
        FileProtocol::Download,
        filename.clone(), // <- Data sent automatically during connection
    ).await?;

    // Stream file content directly to local file using convenient copy method
    let local_filename = format!("downloaded_{}", filename);
    let mut output_file = tokio::fs::File::create(&local_filename).await?;
    
    // Clean copy method - no manual tokio::io::copy needed!
    let bytes_copied = session.copy_to(&mut output_file).await?;
    
    println!("✅ Downloaded {} ({} bytes)", filename, bytes_copied);
    println!("💾 Saved as: {}", local_filename);
    
    Ok(())
}

// Streaming file handler - filename automatically extracted from connection data
async fn file_stream_handler(mut session: fastn_p2p::Session<FileProtocol>, filename: String) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("📂 File request for '{}' from {}", filename, session.peer().id52());
    
    // Security: Only serve files in current directory, no path traversal
    if filename.contains("..") || filename.contains('/') {
        println!("❌ Path traversal attempt blocked: {}", filename);
        return Ok(());
    }
    
    match tokio::fs::File::open(&filename).await {
        Ok(mut file) => {
            println!("📤 Streaming file: {}", filename);
            
            // Clean copy method - no manual tokio::io::copy needed!
            let bytes_sent = session.copy_from(&mut file).await?;
            println!("✅ Sent {} ({} bytes)", filename, bytes_sent);
        }
        Err(e) => {
            println!("❌ File error: {}", e);
            // Could send error info on stream, but keeping simple for now
        }
    }
    
    Ok(())
}
