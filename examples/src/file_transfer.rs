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

#[derive(clap::Parser)]
struct Args {
    #[command(subcommand)]
    mode: Mode,
}

#[derive(clap::Subcommand)]
enum Mode {
    /// Start file server
    Server {
        /// Optional private key
        #[arg(long)]
        key: Option<String>,
    },
    /// Download file from server
    Client {
        /// Target server ID52
        target: String,
        /// Filename to download
        filename: String,
    },
}

#[fastn_context::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = <Args as clap::Parser>::parse();

    match args.mode {
        Mode::Server { key } => run_server(key).await,
        Mode::Client { target, filename } => run_client(target, filename).await,
    }
}

async fn run_server(key: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    let private_key = examples::key_from_str_or_generate(key.as_deref())?;
    
    println!("📁 File server listening on: {}", private_key.id52());
    println!("⚠️  Security: Only files in current directory are served!");
    
    fastn_p2p::listen(private_key)
        .handle_streams(FileProtocol::Download, file_stream_handler)
        .await?;
        
    Ok(())
}

async fn run_client(target: String, filename: String) -> Result<(), Box<dyn std::error::Error>> {
    let private_key = fastn_id52::SecretKey::generate();
    let target_key = examples::parse_peer_id(&target)?;

    println!("📥 Requesting file '{}' from {}", filename, target);

    let mut session = fastn_p2p::client::connect(
        private_key,
        target_key,
        FileProtocol::Download,
    ).await?;

    // Send filename as first line
    session.stdin.write_all(filename.as_bytes()).await?;
    session.stdin.write_all(b"\n").await?;

    // Stream file content directly to local file
    let local_filename = format!("downloaded_{}", filename);
    let mut output_file = tokio::fs::File::create(&local_filename).await?;
    
    // Direct async copy from stream to file - no memory loading!
    let bytes_copied = tokio::io::copy(&mut session.stdout, &mut output_file).await?;
    
    println!("✅ Downloaded {} ({} bytes)", filename, bytes_copied);
    println!("💾 Saved as: {}", local_filename);
    
    Ok(())
}

// Streaming file handler - serves files using direct async I/O copy
async fn file_stream_handler(mut session: fastn_p2p::Session<FileProtocol>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("📂 File streaming session started from {}", session.peer().id52());
    
    // TODO: Read filename from stream (need fastn_net::next_string utility)
    // For now, hardcode a test file
    let filename = "Cargo.toml"; // Use existing file for testing
    
    // Security: Only serve files in current directory, no path traversal
    if filename.contains("..") || filename.contains('/') {
        println!("❌ Path traversal attempt blocked: {}", filename);
        return Ok(());
    }
    
    match tokio::fs::File::open(filename).await {
        Ok(mut file) => {
            println!("📤 Streaming file: {}", filename);
            
            // Direct async copy from file to stream - zero memory copying!
            let bytes_sent = tokio::io::copy(&mut file, &mut session.send).await
                .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })?;
            println!("✅ Sent {} ({} bytes)", filename, bytes_sent);
        }
        Err(e) => {
            println!("❌ File error: {}", e);
            // Could send error info on stream, but keeping simple for now
        }
    }
    
    Ok(())
}