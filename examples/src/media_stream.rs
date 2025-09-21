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
            private_key: _,
            config,
        } => {
            // Always use persistent key for consistent server ID
            let server_key = examples::get_or_create_persistent_key("media_stream");
            let audio_file = config.first().cloned().unwrap_or_else(|| "examples/assets/SerenataGranados.ogg".to_string());
            run_publisher(server_key, audio_file).await
        }
        examples::Client { target, config: _ } => {
            run_subscriber(target).await
        }
    }
}

async fn run_publisher(
    private_key: fastn_p2p::SecretKey,
    audio_file: String,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🎵 Audio Publisher starting...");
    println!("📁 Audio file: {}", audio_file);
    println!("🎧 Publisher listening on: {}", private_key.id52());
    println!("");
    println!("🚀 To connect from another machine, run:");
    println!("   cargo run --bin media_stream -- client {}", private_key.id52());
    println!("");
    
    // Check if audio file exists, if not create a test tone
    if !std::path::Path::new(&audio_file).exists() {
        println!("📝 Creating test audio file: {}", audio_file);
        create_test_audio(&audio_file).await?;
    }
    
    // Show audio info at startup
    if let Ok((_, sample_rate, channels)) = load_audio_file_with_format(&audio_file).await {
        let file_size = std::fs::metadata(&audio_file).map(|m| m.len()).unwrap_or(0);
        println!("📀 Audio file info:");
        println!("   📦 File: {} ({:.1} KB)", audio_file, file_size as f64 / 1024.0);
        println!("   🎵 Format: {}Hz, {} channel(s)", sample_rate, channels);
    }

    fastn_p2p::listen(private_key)
        .handle_streams(MediaProtocol::AudioStream, audio_file, audio_publisher_handler)
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

    // Calculate final metrics
    let total_duration = stats.start_time.unwrap().elapsed().as_secs_f64();
    let avg_throughput_kbps = (stats.bytes_received as f64 * 8.0) / total_duration / 1000.0;
    let packet_loss_rate = if stats.chunks_received > 0 {
        (stats.chunks_dropped as f64 / stats.chunks_received as f64) * 100.0
    } else {
        0.0
    };
    
    println!("");
    println!("📊 Client Streaming Metrics:");
    println!("   ⏱️  Total duration: {:.1}s", total_duration);
    println!("   📦 Chunks received: {}", stats.chunks_received);
    println!("   💾 Data received: {:.1} KB", stats.bytes_received as f64 / 1024.0);
    println!("   🚀 Average throughput: {:.0} kbps", avg_throughput_kbps);
    println!("   📉 Packet loss: {:.2}%", packet_loss_rate);
    println!("   🔊 Audio quality: {}", if packet_loss_rate < 1.0 { "Excellent" } else if packet_loss_rate < 5.0 { "Good" } else { "Poor" });
    
    if stats.chunks_dropped > 0 {
        println!("   ⚠️  {} chunks dropped - may cause audio gaps", stats.chunks_dropped);
    }
    
    Ok(())
}

// Audio publisher handler - streams audio chunks to subscriber
async fn audio_publisher_handler(
    mut session: fastn_p2p::Session<MediaProtocol>,
    _data: (),
    audio_file: String,
) -> Result<(), MediaError> {
    println!("🔊 New subscriber connected: {}", session.peer().id52());
    
    // Read and decode audio file to get actual audio format
    let (audio_data, sample_rate, channels) = load_audio_file_with_format(&audio_file).await?;
    let mut stats = StreamStats::default();
    stats.start_time = Some(Instant::now());
    
    // Stream audio chunks at regular intervals
    let chunk_size = 4096; // 4KB chunks
    let mut sequence = 0u64;
    let stream_start = Instant::now();
    
    // Calculate proper timing based on ACTUAL audio format
    let bytes_per_second = sample_rate as u64 * channels as u64 * 2; // 2 bytes per sample (i16)
    let chunk_duration_ms = (chunk_size as f64 / bytes_per_second as f64 * 1000.0) as u64;
    
    println!("🎵 Streaming config:");
    println!("   🔊 Format: {}Hz, {} channel(s), 16-bit", sample_rate, channels);
    println!("   📦 Chunk size: {} bytes = {:.1}ms of audio", chunk_size, chunk_duration_ms);
    println!("   ⏱️  Expected stream duration: {:.1}s", audio_data.len() as f64 / bytes_per_second as f64);
    
    let mut interval = interval(Duration::from_millis(chunk_duration_ms.max(10))); // At least 10ms
    
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
    
    // Calculate server-side metrics
    let total_duration = stats.start_time.unwrap().elapsed().as_secs_f64();
    let avg_throughput_kbps = (stats.bytes_sent as f64 * 8.0) / total_duration / 1000.0;
    
    println!("");
    println!("📊 Server Streaming Metrics:");
    println!("   ⏱️  Stream duration: {:.1}s", total_duration);
    println!("   📦 Chunks sent: {}", stats.chunks_sent);
    println!("   💾 Data sent: {:.1} KB", stats.bytes_sent as f64 / 1024.0);
    println!("   🚀 Average throughput: {:.0} kbps", avg_throughput_kbps);
    println!("   🎵 Stream completed successfully");
    
    Ok(())
}

// Load audio file (MP3/OGG) and decode to PCM audio data with format info
async fn load_audio_file_with_format(filename: &str) -> Result<(Vec<u8>, u32, u16), MediaError> {
    println!("📁 Loading and decoding audio file: {}", filename);
    
    let file_data = tokio::fs::read(filename).await
        .map_err(|_| MediaError::FileNotFound(filename.to_string()))?;
    
    // Try different decoders based on file extension
    let extension = std::path::Path::new(filename)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_lowercase();
    
    let (pcm_data, sample_rate, channels) = match extension.as_str() {
        "mp3" => decode_mp3(&file_data)?,
        "ogg" => decode_with_symphonia(&file_data)?,
        _ => {
            // Try symphonia first (supports many formats), fallback to mp3
            decode_with_symphonia(&file_data)
                .or_else(|_| decode_mp3(&file_data))?
        }
    };
    
    // Calculate and display audio info
    let file_size_kb = file_data.len() as f64 / 1024.0;
    let pcm_size_kb = pcm_data.len() as f64 / 1024.0;
    let duration = pcm_data.len() as f64 / (sample_rate as f64 * channels as f64 * 2.0); // 2 bytes per sample
    let bitrate = (file_size_kb * 8.0) / duration;
    
    println!("✅ Audio Decoded:");
    println!("   📦 File size: {:.1} KB", file_size_kb);
    println!("   🔊 PCM size: {:.1} KB", pcm_size_kb);
    println!("   ⏱️  Duration: {:.1}s", duration);
    println!("   🎵 Sample rate: {} Hz", sample_rate);
    println!("   📻 Channels: {}", channels);
    println!("   📡 Bitrate: {:.0} kbps", bitrate);
    
    Ok((pcm_data, sample_rate, channels))
}

// Decode MP3 using minimp3
fn decode_mp3(file_data: &[u8]) -> Result<(Vec<u8>, u32, u16), MediaError> {
    let mut decoder = minimp3::Decoder::new(std::io::Cursor::new(file_data));
    let mut pcm_data = Vec::new();
    let mut sample_rate = 44100;
    let mut channels = 2;
    
    loop {
        match decoder.next_frame() {
            Ok(frame) => {
                sample_rate = frame.sample_rate as u32;
                channels = frame.channels as u16;
                // Convert i16 samples to bytes
                for sample in frame.data {
                    pcm_data.extend_from_slice(&sample.to_le_bytes());
                }
            }
            Err(minimp3::Error::Eof) => break,
            Err(e) => {
                return Err(MediaError::DecodeError(format!("MP3 decode error: {:?}", e)));
            }
        }
    }
    
    if pcm_data.is_empty() {
        return Err(MediaError::DecodeError("No MP3 audio data decoded".to_string()));
    }
    
    Ok((pcm_data, sample_rate, channels))
}

// Decode OGG/Vorbis and other formats using symphonia
fn decode_with_symphonia(file_data: &[u8]) -> Result<(Vec<u8>, u32, u16), MediaError> {
    use symphonia::core::audio::{AudioBufferRef, Signal};
    use symphonia::core::codecs::DecoderOptions;
    use symphonia::core::formats::FormatOptions;
    use symphonia::core::io::MediaSourceStream;
    use symphonia::core::meta::MetadataOptions;
    use symphonia::core::probe::Hint;
    
    let file_data_owned = file_data.to_vec();
    let cursor = std::io::Cursor::new(file_data_owned);
    let mss = MediaSourceStream::new(Box::new(cursor), Default::default());
    
    let mut hint = Hint::new();
    // Let symphonia probe the format
    
    let meta_opts = MetadataOptions::default();
    let fmt_opts = FormatOptions::default();
    
    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &fmt_opts, &meta_opts)
        .map_err(|e| MediaError::DecodeError(format!("Format probe failed: {:?}", e)))?;
    
    let mut format = probed.format;
    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL)
        .ok_or_else(|| MediaError::DecodeError("No supported audio tracks found".to_string()))?;
    
    let dec_opts = DecoderOptions::default();
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &dec_opts)
        .map_err(|e| MediaError::DecodeError(format!("Decoder creation failed: {:?}", e)))?;
    
    let track_id = track.id;
    let mut pcm_data = Vec::new();
    let mut sample_rate = 44100;
    let mut channels = 2;
    
    // Decode all packets
    loop {
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(symphonia::core::errors::Error::ResetRequired) => {
                // The track list has been changed. Re-examine it and create a new set of decoders,
                // then restart the decode loop. This is an advanced feature that most programs
                // will never encounter.
                unimplemented!();
            }
            Err(symphonia::core::errors::Error::IoError(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                // End of stream
                break;
            }
            Err(e) => {
                return Err(MediaError::DecodeError(format!("Packet read error: {:?}", e)));
            }
        };
        
        // Only decode packets for our track
        if packet.track_id() != track_id {
            continue;
        }
        
        match decoder.decode(&packet) {
            Ok(decoded) => {
                // Extract sample rate and channel info
                sample_rate = decoded.spec().rate;
                channels = decoded.spec().channels.count() as u16;
                
                // Convert audio buffer to i16 PCM
                match decoded {
                    AudioBufferRef::F32(buf) => {
                        for &sample in buf.chan(0) {
                            let sample_i16 = (sample * 32767.0).clamp(-32767.0, 32767.0) as i16;
                            pcm_data.extend_from_slice(&sample_i16.to_le_bytes());
                        }
                        // Handle additional channels if stereo
                        if buf.spec().channels.count() > 1 {
                            for &sample in buf.chan(1) {
                                let sample_i16 = (sample * 32767.0).clamp(-32767.0, 32767.0) as i16;
                                pcm_data.extend_from_slice(&sample_i16.to_le_bytes());
                            }
                        }
                    }
                    AudioBufferRef::U16(buf) => {
                        for &sample in buf.chan(0) {
                            let sample_i16 = (sample as i32 - 32768) as i16;
                            pcm_data.extend_from_slice(&sample_i16.to_le_bytes());
                        }
                        if buf.spec().channels.count() > 1 {
                            for &sample in buf.chan(1) {
                                let sample_i16 = (sample as i32 - 32768) as i16;
                                pcm_data.extend_from_slice(&sample_i16.to_le_bytes());
                            }
                        }
                    }
                    AudioBufferRef::S16(buf) => {
                        for &sample in buf.chan(0) {
                            pcm_data.extend_from_slice(&sample.to_le_bytes());
                        }
                        if buf.spec().channels.count() > 1 {
                            for &sample in buf.chan(1) {
                                pcm_data.extend_from_slice(&sample.to_le_bytes());
                            }
                        }
                    }
                    _ => {
                        return Err(MediaError::DecodeError("Unsupported audio format".to_string()));
                    }
                }
            }
            Err(symphonia::core::errors::Error::IoError(_)) => {
                // End of stream
                break;
            }
            Err(symphonia::core::errors::Error::DecodeError(_)) => {
                // Decode error, try to continue
                continue;
            }
            Err(e) => {
                return Err(MediaError::DecodeError(format!("Decode error: {:?}", e)));
            }
        }
    }
    
    if pcm_data.is_empty() {
        return Err(MediaError::DecodeError("No audio data decoded".to_string()));
    }
    
    Ok((pcm_data, sample_rate, channels))
}


// Create a test audio file (simple sine wave as MP3)
async fn create_test_audio(filename: &str) -> Result<(), MediaError> {
    println!("🎼 Generating test audio (musical scale)...");
    
    // Generate a musical scale instead of single tone for better quality testing
    let sample_rate = 44100;
    let duration = 10; // 10 seconds total
    let notes = [261.63, 293.66, 329.63, 349.23, 392.00, 440.00, 493.88, 523.25]; // C major scale
    
    let mut samples = Vec::new();
    let note_duration = duration as f32 / notes.len() as f32; // Each note duration
    
    for i in 0..(sample_rate * duration) {
        let t = i as f32 / sample_rate as f32;
        let note_index = (t / note_duration) as usize % notes.len();
        let frequency = notes[note_index];
        
        // Create a more pleasant sound with some harmonics
        let fundamental = (2.0 * std::f32::consts::PI * frequency * t).sin();
        let harmonic = 0.3 * (2.0 * std::f32::consts::PI * frequency * 2.0 * t).sin();
        let sample = (fundamental + harmonic) * 0.7; // Reduce volume slightly
        
        let sample_i16 = (sample * 32767.0) as i16;
        samples.extend_from_slice(&sample_i16.to_le_bytes());
    }
    
    // For simplicity, save as raw PCM data with .mp3 extension
    // In a real implementation, you'd encode this as actual MP3
    tokio::fs::write(filename, &samples).await?;
    
    println!("✅ Created test audio: C major scale ({} bytes, {}s)", samples.len(), duration);
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