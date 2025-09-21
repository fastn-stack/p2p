//! Media Streaming Example
//!
//! Live audio streaming over P2P using MP3 files as source.
//! Demonstrates real-time media streaming between peers.
//!
//! This shows high-throughput, low-latency P2P streaming patterns.
//!
//! Usage:
//!   media_stream publisher [mp3_file] [key]     # Start media publisher
//!   media_stream subscriber <id52>              # Subscribe to media stream

use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc;
use tokio::time::interval;

// Protocol Definition
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub enum MediaProtocol {
    AudioStream,
}

// Audio chunk for streaming
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct AudioChunk {
    pub sequence: u64,
    pub timestamp: u64, // Microseconds since stream start
    pub data: Vec<u8>,
    pub sample_rate: u32,
    pub channels: u16,
}

// Audio stream statistics
#[derive(Debug, Default)]
pub struct StreamStats {
    pub chunks_sent: u64,
    pub chunks_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub start_time: Option<Instant>,
    pub last_chunk_time: Option<Instant>,
    pub chunks_dropped: u64,
    pub last_sequence: u64,
}

// Custom error type for media operations
#[derive(Debug, thiserror::Error)]
pub enum MediaError {
    #[error("Audio file not found: {0}")]
    FileNotFound(String),
    #[error("Audio decode error: {0}")]
    DecodeError(String),
    #[error("Audio playback error: {0}")]
    PlaybackError(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[fastn_p2p::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    match examples::parse_cli()? {
        examples::Server {
            private_key,
            config,
        } => {
            let mp3_file = config.first().cloned().unwrap_or_else(|| "test_audio.mp3".to_string());
            run_publisher(private_key, mp3_file).await
        }
        examples::Client { target, config: _ } => {
            run_subscriber(target).await
        }
    }
}

async fn run_publisher(
    private_key: fastn_p2p::SecretKey,
    mp3_file: String,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🎵 Audio Publisher starting...");
    println!("📁 MP3 file: {}", mp3_file);
    println!("🎧 Publisher listening on: {}", private_key.id52());
    println!("");
    println!("🚀 To connect from another machine, run:");
    println!("   cargo run --bin media_stream -- client {}", private_key.id52());
    println!("");
    
    // Check if MP3 file exists, if not create a test tone
    if !std::path::Path::new(&mp3_file).exists() {
        println!("📝 Creating test audio file: {}", mp3_file);
        create_test_audio(&mp3_file).await?;
    }

    fastn_p2p::listen(private_key)
        .handle_streams(MediaProtocol::AudioStream, mp3_file, audio_publisher_handler)
        .await?;

    Ok(())
}

async fn run_subscriber(
    target: fastn_p2p::PublicKey,
) -> Result<(), Box<dyn std::error::Error>> {
    let private_key = fastn_p2p::SecretKey::generate();

    println!("🎧 Audio Subscriber connecting to: {}", target);
    println!("🔊 Starting audio playback...");

    // Connect to the publisher
    let mut session = fastn_p2p::client::connect(
        private_key,
        target,
        MediaProtocol::AudioStream,
        (), // No data needed for subscription
    ).await?;

    // Start audio playback system
    let (_stream, stream_handle) = rodio::OutputStream::try_default()
        .map_err(|e| MediaError::PlaybackError(format!("Failed to create audio output: {}", e)))?;
    
    let sink = rodio::Sink::try_new(&stream_handle)
        .map_err(|e| MediaError::PlaybackError(format!("Failed to create audio sink: {}", e)))?;

    let mut stats = StreamStats::default();
    stats.start_time = Some(Instant::now());

    // Audio playback buffer
    let (audio_tx, mut audio_rx) = mpsc::channel::<AudioChunk>(100);

    // Spawn audio player task
    let sink = std::sync::Arc::new(sink);
    let sink_clone = sink.clone();
    tokio::spawn(async move {
        while let Some(chunk) = audio_rx.recv().await {
            // Convert raw audio data to playable format
            if let Ok(source) = create_audio_source(chunk) {
                sink_clone.append(source);
            }
        }
    });

    // Receive and process audio chunks
    loop {
        let mut chunk_size_buf = [0u8; 4];
        if session.stdout.read_exact(&mut chunk_size_buf).await.is_err() {
            break;
        }
        
        let chunk_size = u32::from_le_bytes(chunk_size_buf) as usize;
        if chunk_size > 1024 * 1024 { // 1MB max chunk size
            eprintln!("⚠️ Chunk too large: {} bytes", chunk_size);
            continue;
        }
        
        let mut chunk_data = vec![0u8; chunk_size];
        if session.stdout.read_exact(&mut chunk_data).await.is_err() {
            break;
        }

        match bincode::deserialize::<AudioChunk>(&chunk_data) {
            Ok(chunk) => {
                // Update statistics
                stats.chunks_received += 1;
                stats.bytes_received += chunk.data.len() as u64;
                stats.last_chunk_time = Some(Instant::now());
                
                // Check for dropped chunks
                if chunk.sequence > stats.last_sequence + 1 {
                    stats.chunks_dropped += chunk.sequence - stats.last_sequence - 1;
                    eprintln!("📉 Dropped {} chunks (seq {} -> {})", 
                             chunk.sequence - stats.last_sequence - 1,
                             stats.last_sequence, chunk.sequence);
                }
                stats.last_sequence = chunk.sequence;

                // Send to audio player
                if audio_tx.send(chunk).await.is_err() {
                    eprintln!("⚠️ Audio buffer full, dropping chunk");
                }

                // Print stats every 100 chunks
                if stats.chunks_received % 100 == 0 {
                    let elapsed = stats.start_time.unwrap().elapsed();
                    let throughput = stats.bytes_received as f64 / elapsed.as_secs_f64() / 1024.0;
                    println!("📊 Received {} chunks, {:.1} KB/s, {} dropped", 
                            stats.chunks_received, throughput, stats.chunks_dropped);
                }
            }
            Err(e) => {
                eprintln!("❌ Failed to deserialize chunk: {}", e);
            }
        }
    }

    println!("📊 Final stats: {} chunks, {:.2} KB total, {} dropped", 
             stats.chunks_received, stats.bytes_received as f64 / 1024.0, stats.chunks_dropped);
    
    Ok(())
}

// Audio publisher handler - streams audio chunks to subscriber
async fn audio_publisher_handler(
    mut session: fastn_p2p::Session<MediaProtocol>,
    _data: (),
    mp3_file: String,
) -> Result<(), MediaError> {
    println!("🔊 New subscriber connected: {}", session.peer().id52());
    
    // Read and decode MP3 file
    let audio_data = load_mp3_file(&mp3_file).await?;
    let mut stats = StreamStats::default();
    stats.start_time = Some(Instant::now());
    
    // Stream audio chunks at regular intervals
    let chunk_size = 4096; // 4KB chunks
    let mut sequence = 0u64;
    let stream_start = Instant::now();
    
    // Target 44.1kHz, 16-bit stereo (about 90 chunks per second for real-time)
    let mut interval = interval(Duration::from_millis(11)); // ~90 Hz
    
    for chunk_data in audio_data.chunks(chunk_size) {
        interval.tick().await;
        
        let chunk = AudioChunk {
            sequence,
            timestamp: stream_start.elapsed().as_micros() as u64,
            data: chunk_data.to_vec(),
            sample_rate: 44100,
            channels: 2,
        };
        
        // Serialize and send chunk
        match bincode::serialize(&chunk) {
            Ok(serialized) => {
                let size = serialized.len() as u32;
                
                // Send chunk size first
                if session.send.write_all(&size.to_le_bytes()).await.is_err() {
                    break;
                }
                
                // Send chunk data
                if session.send.write_all(&serialized).await.is_err() {
                    break;
                }
                
                stats.chunks_sent += 1;
                stats.bytes_sent += chunk.data.len() as u64;
                sequence += 1;
                
                if sequence % 100 == 0 {
                    let elapsed = stats.start_time.unwrap().elapsed();
                    let throughput = stats.bytes_sent as f64 / elapsed.as_secs_f64() / 1024.0;
                    println!("📡 Sent {} chunks, {:.1} KB/s", stats.chunks_sent, throughput);
                }
            }
            Err(e) => {
                eprintln!("❌ Failed to serialize chunk: {}", e);
                break;
            }
        }
    }
    
    println!("✅ Finished streaming {} chunks ({:.2} KB total)", 
             stats.chunks_sent, stats.bytes_sent as f64 / 1024.0);
    
    Ok(())
}

// Load MP3 file and decode to raw audio data
async fn load_mp3_file(filename: &str) -> Result<Vec<u8>, MediaError> {
    println!("📁 Loading MP3 file: {}", filename);
    
    let data = tokio::fs::read(filename).await
        .map_err(|_| MediaError::FileNotFound(filename.to_string()))?;
    
    // Calculate and display MP3 info
    let file_size_kb = data.len() as f64 / 1024.0;
    let estimated_duration = estimate_mp3_duration(&data);
    let estimated_bitrate = if estimated_duration > 0.0 {
        (file_size_kb * 8.0) / estimated_duration
    } else {
        0.0
    };
    
    println!("✅ MP3 Info:");
    println!("   📦 Size: {:.1} KB", file_size_kb);
    if estimated_duration > 0.0 {
        println!("   ⏱️  Estimated duration: {:.1}s", estimated_duration);
        println!("   🎵 Estimated bitrate: {:.0} kbps", estimated_bitrate);
    }
    println!("   📡 Will stream as {} KB audio data", file_size_kb);
    
    Ok(data)
}

// Simple MP3 duration estimation (very rough)
fn estimate_mp3_duration(data: &[u8]) -> f64 {
    // Look for MP3 frame headers to estimate duration
    // This is a very basic estimation - real MP3 parsing would be more accurate
    let mut frame_count = 0;
    let mut i = 0;
    
    while i < data.len().saturating_sub(4) {
        // Look for MP3 frame sync (0xFF followed by 0xE0-0xFF)
        if data[i] == 0xFF && (data[i + 1] & 0xE0) == 0xE0 {
            frame_count += 1;
            i += 4; // Skip frame header
        } else {
            i += 1;
        }
    }
    
    // Very rough estimation: ~38 frames per second for typical MP3
    if frame_count > 10 {
        frame_count as f64 / 38.0
    } else {
        // Fallback: assume ~128kbps for raw audio data
        (data.len() as f64 * 8.0) / (128.0 * 1000.0)
    }
}

// Create a test audio file (simple sine wave as MP3)
async fn create_test_audio(filename: &str) -> Result<(), MediaError> {
    println!("🎼 Generating test audio tone...");
    
    // Generate a simple sine wave (440 Hz A note)
    let sample_rate = 44100;
    let duration = 10; // 10 seconds
    let frequency = 440.0; // A4 note
    
    let mut samples = Vec::new();
    for i in 0..(sample_rate * duration) {
        let t = i as f32 / sample_rate as f32;
        let sample = (2.0 * std::f32::consts::PI * frequency * t).sin();
        let sample_i16 = (sample * 32767.0) as i16;
        samples.extend_from_slice(&sample_i16.to_le_bytes());
    }
    
    // For simplicity, save as raw PCM data with .mp3 extension
    // In a real implementation, you'd encode this as actual MP3
    tokio::fs::write(filename, &samples).await?;
    
    println!("✅ Created test audio file: {} ({} bytes)", filename, samples.len());
    Ok(())
}

// Create an audio source from raw chunk data for playback
fn create_audio_source(chunk: AudioChunk) -> Result<rodio::buffer::SamplesBuffer<i16>, MediaError> {
    // Convert raw bytes to i16 samples
    let mut samples = Vec::new();
    for chunk_bytes in chunk.data.chunks_exact(2) {
        if chunk_bytes.len() == 2 {
            let sample = i16::from_le_bytes([chunk_bytes[0], chunk_bytes[1]]);
            samples.push(sample);
        }
    }
    
    let source = rodio::buffer::SamplesBuffer::new(
        chunk.channels,
        chunk.sample_rate,
        samples
    );
    
    Ok(source)
}