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
    println!("🎵 Stream Server V2 starting...");
    println!("🎧 Server listening on: {}", private_key.id52());
    println!("");
    println!("🚀 To connect from another machine, run:");
    println!("   cargo run --bin media_stream_v2 -- client {}", private_key.id52());
    println!("");
    
    // Create stream provider
    let provider = SimpleAudioProvider::new(audio_file).await?;
    
    // Start server (TODO: Need to wire up handlers properly with fastn-p2p)
    println!("📡 Server ready to serve audio streams...");
    
    // For now, just keep server alive
    tokio::signal::ctrl_c().await?;
    println!("👋 Server shutting down...");
    
    Ok(())
}

/// Run client with clean stream access
async fn run_client(
    target: fastn_p2p::PublicKey,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🎧 Stream Client V2 connecting to: {}", target);
    
    // Create stream client
    let stream_client = StreamClient::new(target);
    
    // Open the audio stream
    let stream = stream_client.open_stream("audio_stream").await?;
    println!("✅ Opened stream: {} with {} tracks", stream.name, stream.list_tracks().len());
    
    // Get audio track
    let audio_track = stream.get_track("audio")
        .ok_or("Audio track not found in stream")?;
    
    println!("📊 Audio track: {} bytes", audio_track.size_bytes);
    
    // Simple test: read first chunk
    let chunk_size = 32768; // 32KB
    let chunk_data = stream_client.read_track_range("audio_stream", "audio", 0, chunk_size).await?;
    println!("📥 Read first chunk: {} bytes", chunk_data.len());
    
    // TODO: Add full playback loop with buffering
    // TODO: Add interactive controls (SPACE pause/resume)
    // TODO: Add audio decoding + rodio playback
    
    Ok(())
}

/// Simple audio stream provider implementation
struct SimpleAudioProvider {
    audio_file: String,
    audio_data: Vec<u8>,
}

impl SimpleAudioProvider {
    async fn new(audio_file: String) -> Result<Self, Box<dyn std::error::Error>> {
        // TODO: Load and decode audio file using examples::audio_decoder
        let (audio_data, _sample_rate, _channels) = examples::audio_decoder::decode_audio_file(&audio_file).await
            .map_err(|e| format!("Failed to decode audio: {}", e))?;
        
        Ok(Self {
            audio_file,
            audio_data,
        })
    }
}

impl StreamProvider for SimpleAudioProvider {
    async fn resolve_stream(&self, stream_name: &str) -> Option<ServerStream> {
        // TODO: If stream_name == "audio_stream", return stream with single audio track
        if stream_name == "audio_stream" {
            let mut stream = ServerStream::new(stream_name.to_string());
            stream.add_track("audio".to_string(), self.audio_data.len() as u64);
            Some(stream)
        } else {
            None
        }
    }
    
    async fn read_track_range(&self, _stream_name: &str, _track_name: &str, start: u64, length: u64) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // TODO: Check bounds (start + length <= audio_data.len())
        let end = std::cmp::min(start + length, self.audio_data.len() as u64) as usize;
        let start = start as usize;
        
        if start >= self.audio_data.len() {
            return Err("Start position out of bounds".into());
        }
        
        Ok(self.audio_data[start..end].to_vec())
    }
}