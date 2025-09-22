//! Clean Stream-Based Media Example
//!
//! Demonstrates the new clean streaming API:
//! - Stream provider trait (app implements)
//! - Clean client/server separation
//! - No embedded connection info in types

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

/// Run server with stream provider
async fn run_server(
    private_key: fastn_p2p::SecretKey,
    audio_file: String,
) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Print "Stream Server starting..."
    // TODO: Create SimpleAudioProvider::new(audio_file) - implements StreamProvider trait
    // TODO: Setup fastn_p2p::listen() with GET_STREAM handler using provider
    // TODO: Setup fastn_p2p::listen() with READ_TRACK_RANGE handler using provider
    // TODO: Print server ID and connection command
    // TODO: Start listening
    todo!()
}

/// Run client with clean stream access
async fn run_client(
    target: fastn_p2p::PublicKey,
) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: Print "Stream Client connecting to: {target}"
    // TODO: Create StreamClient::new(target)
    // TODO: Call client.open_stream("audio_stream") to get ClientStream
    // TODO: Get audio track from stream
    // TODO: Start playback loop using client.read_track_range() calls
    // TODO: Add interactive controls (SPACE pause/resume)
    todo!()
}

/// Simple audio stream provider implementation
struct SimpleAudioProvider {
    audio_file: String,
    audio_data: Vec<u8>,
}

impl SimpleAudioProvider {
    async fn new(audio_file: String) -> Result<Self, Box<dyn std::error::Error>> {
        // TODO: Load and decode audio file using examples::audio_decoder
        // TODO: Store audio_data for serving
        // TODO: Return SimpleAudioProvider instance
        todo!()
    }
}

impl StreamProvider for SimpleAudioProvider {
    async fn resolve_stream(&self, stream_name: &str) -> Option<ServerStream> {
        // TODO: If stream_name == "audio_stream", return stream with single audio track
        // TODO: Track size = self.audio_data.len()
        // TODO: Return None for unknown streams
        todo!()
    }
    
    async fn read_track_range(&self, _stream_name: &str, _track_name: &str, start: u64, length: u64) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // TODO: Check bounds (start + length <= audio_data.len())
        // TODO: Return self.audio_data[start..start+length].to_vec()
        // TODO: Handle out of bounds errors
        todo!()
    }
}