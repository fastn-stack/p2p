//! Client-Driven Media Streaming V2 - Main Entry Point
//!
//! Orchestrates the modular streaming components:
//! - Uses current enum protocol approach 
//! - Clean separation: protocol, server, client, UI
//! - Interactive SPACE pause/resume controls

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

/// Run audio server - loads audio file and handles client requests
async fn run_server(
    private_key: fastn_p2p::SecretKey,
    audio_file: String,
) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Print "Audio Server V2 starting..."
    // TODO: Print server ID and connection command
    // TODO: Create AudioServer::new(&audio_file) - loads and decodes audio
    // TODO: Setup fastn_p2p::listen() with AudioProtocol::GetInfo handler
    // TODO: Setup fastn_p2p::listen() with AudioProtocol::RequestChunk handler  
    // TODO: Start listening for requests
    todo!()
}

/// Run audio client - connects, buffers, and plays audio with interactive controls
async fn run_client(
    target: fastn_p2p::PublicKey,
) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Print "Audio Client V2 connecting to: {target}"
    // TODO: Create AudioClient::connect(target) - gets stream info
    // TODO: Create StreamingUI::new(client) - setup audio playback
    // TODO: Call ui.start_streaming() - starts all background tasks
    todo!()
}