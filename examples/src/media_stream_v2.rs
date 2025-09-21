//! Client-Driven Media Streaming V2
//!
//! Clean modular implementation with:
//! - Client-controlled buffering
//! - Interactive play/pause controls  
//! - Request/response protocol
//! - Separated concerns: protocol, server, client, UI

mod streaming;

use streaming::*;

#[fastn_p2p::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    match examples::parse_cli()? {
        examples::Server {
            private_key: _,
            config,
        } => {
            let server_key = examples::get_or_create_persistent_key("media_stream_v2");
            let audio_file = config.first().cloned().unwrap_or_else(|| 
                "examples/assets/SerenataGranados.ogg".to_string()
            );
            run_server(server_key, audio_file).await
        }
        examples::Client { target, config: _ } => {
            run_client(target).await
        }
    }
}

async fn run_server(
    private_key: fastn_p2p::SecretKey,
    audio_file: String,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🎵 Audio Server V2 starting...");
    println!("🎧 Server listening on: {}", private_key.id52());
    println!("");
    println!("🚀 To connect from another machine, run:");
    println!("   cargo run --bin media_stream_v2 -- client {}", private_key.id52());
    println!("");
    
    // Load audio data once
    let audio_server = AudioServer::new(&audio_file).await?;
    
    // Start request handler
    fastn_p2p::listen(private_key)
        .handle_requests(StreamingProtocol::AudioV2, |request| {
            let server = audio_server.clone();
            async move { server::handle_request(request, server).await }
        })
        .await?;
    
    Ok(())
}

async fn run_client(
    target: fastn_p2p::PublicKey,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🎧 Audio Client V2 connecting to: {}", target);
    
    // Connect and get stream info
    let client = AudioClient::connect(target).await?;
    
    // Create UI and start streaming
    let ui = StreamingUI::new(client).await?;
    ui.start_streaming().await?;
    
    Ok(())
}