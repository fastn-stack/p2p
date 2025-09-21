//! Client-Driven Media Streaming Example
//!
//! Pull-based audio streaming with client-controlled buffering and play/pause.
//! Client requests chunks when buffer is low, server responds on-demand.
//!
//! Usage:
//!   media_stream_v2 server [audio_file]    # Start audio server
//!   media_stream_v2 client <id52>          # Connect with interactive controls

use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, Mutex};
use tokio::time::sleep;

// Protocol Definition - Client-driven pull-based streaming
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub enum MediaProtocolV2 {
    AudioStreamV2,
}

// Client requests
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum StreamRequest {
    GetStreamInfo,
    RequestChunk { chunk_id: u64 },
    Stop,
}

// Server responses  
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum StreamResponse {
    StreamInfo {
        total_chunks: u64,
        chunk_size: usize,
        sample_rate: u32,
        channels: u16,
        duration_seconds: f64,
    },
    AudioChunk {
        chunk_id: u64,
        data: Vec<u8>,
        is_last: bool,
    },
    EndOfStream,
    Error(String),
}

// Audio buffer manager for client
#[derive(Debug)]
struct AudioBuffer {
    chunks: VecDeque<Vec<u8>>,
    target_buffer_ms: u64,
    current_buffer_ms: u64,
    chunk_duration_ms: u64,
    playing: bool,
}

impl AudioBuffer {
    fn new(target_buffer_ms: u64, chunk_duration_ms: u64) -> Self {
        Self {
            chunks: VecDeque::new(),
            target_buffer_ms,
            current_buffer_ms: 0,
            chunk_duration_ms,
            playing: true,
        }
    }
    
    fn needs_data(&self) -> bool {
        self.playing && self.current_buffer_ms < self.target_buffer_ms
    }
    
    fn add_chunk(&mut self, data: Vec<u8>) {
        self.chunks.push_back(data);
        self.current_buffer_ms += self.chunk_duration_ms;
    }
    
    fn get_chunk(&mut self) -> Option<Vec<u8>> {
        if let Some(chunk) = self.chunks.pop_front() {
            self.current_buffer_ms = self.current_buffer_ms.saturating_sub(self.chunk_duration_ms);
            Some(chunk)
        } else {
            None
        }
    }
    
    fn pause(&mut self) {
        self.playing = false;
    }
    
    fn resume(&mut self) {
        self.playing = true;
    }
}

#[fastn_p2p::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    match examples::parse_cli()? {
        examples::Server {
            private_key: _,
            config,
        } => {
            let server_key = examples::get_or_create_persistent_key("media_stream_v2");
            let audio_file = config.first().cloned().unwrap_or_else(|| "examples/assets/SerenataGranados.ogg".to_string());
            run_audio_server(server_key, audio_file).await
        }
        examples::Client { target, config: _ } => {
            run_audio_client(target).await
        }
    }
}

async fn run_audio_server(
    private_key: fastn_p2p::SecretKey,
    audio_file: String,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🎵 Audio Server starting...");
    println!("📁 Audio file: {}", audio_file);
    
    // Decode audio file once at startup
    println!("🔍 Pre-loading audio file...");
    let decode_start = Instant::now();
    let (audio_data, sample_rate, channels) = load_audio_file_with_format(&audio_file).await
        .map_err(|e| format!("Failed to load audio: {}", e))?;
    let decode_time = decode_start.elapsed();
    
    let duration = audio_data.len() as f64 / (sample_rate as f64 * channels as f64 * 2.0);
    println!("✅ Audio loaded (+{:.3}s): {:.1}s, {}Hz, {} ch", 
             decode_time.as_secs_f64(), duration, sample_rate, channels);
    
    println!("🎧 Server listening on: {}", private_key.id52());
    println!("");
    println!("🚀 To connect from another machine, run:");
    println!("   cargo run --bin media_stream_v2 -- client {}", private_key.id52());
    println!("");

    // Create server state
    let server_state = AudioServerState {
        audio_data,
        sample_rate,
        channels,
        duration,
        chunk_size: 262144, // 256KB chunks
    };

    fastn_p2p::listen(private_key)
        .handle_requests(MediaProtocolV2::AudioStreamV2, move |request| handle_audio_request(request, server_state.clone()))
        .await?;

    Ok(())
}

#[derive(Clone)]
struct AudioServerState {
    audio_data: Vec<u8>,
    sample_rate: u32,
    channels: u16,
    duration: f64,
    chunk_size: usize,
}

async fn run_audio_client(
    target: fastn_p2p::PublicKey,
) -> Result<(), Box<dyn std::error::Error>> {
    let private_key = fastn_p2p::SecretKey::generate();
    let start_time = Instant::now();

    println!("🎧 Audio Client connecting to: {}", target);
    println!("🔍 Establishing P2P connection...");

    // Get stream info first
    let stream_info: StreamResponse = fastn_p2p::client::call(
        private_key.clone(),
        target,
        MediaProtocolV2::AudioStreamV2,
        StreamRequest::GetStreamInfo,
    ).await?;

    let (total_chunks, chunk_size, sample_rate, channels, duration) = match stream_info {
        StreamResponse::StreamInfo { total_chunks, chunk_size, sample_rate, channels, duration_seconds } => {
            println!("✅ Stream info received (+{:.3}s)", start_time.elapsed().as_secs_f64());
            println!("📊 Stream: {:.1}s, {}Hz, {} ch, {} chunks", 
                     duration_seconds, sample_rate, channels, total_chunks);
            (total_chunks, chunk_size, sample_rate, channels, duration_seconds)
        }
        _ => return Err("Failed to get stream info".into()),
    };

    // Setup audio playback
    let (_stream, stream_handle) = rodio::OutputStream::try_default()
        .map_err(|e| format!("Failed to create audio output: {}", e))?;
    let sink = rodio::Sink::try_new(&stream_handle)
        .map_err(|e| format!("Failed to create audio sink: {}", e))?;

    // Calculate buffering parameters
    let chunk_duration_ms = (chunk_size as f64 / (sample_rate as f64 * channels as f64 * 2.0) * 1000.0) as u64;
    let target_buffer_ms = 3000; // 3 seconds of buffering
    let buffer = Arc::new(Mutex::new(AudioBuffer::new(target_buffer_ms, chunk_duration_ms)));

    println!("🔧 Audio system ready (+{:.3}s)", start_time.elapsed().as_secs_f64());
    println!("📦 Chunk size: {}KB = {:.1}s of audio", chunk_size / 1024, chunk_duration_ms as f64 / 1000.0);
    println!("🔊 Target buffer: {:.1}s", target_buffer_ms as f64 / 1000.0);
    println!("💡 Press SPACE to pause/resume, 'q' to quit");

    // Start chunk fetcher task
    let fetch_buffer = buffer.clone();
    let fetch_target = target;
    let fetch_private_key = private_key.clone();
    tokio::spawn(async move {
        let mut next_chunk_id = 0u64;
        
        loop {
            let needs_data = {
                let buffer_guard = fetch_buffer.lock().await;
                buffer_guard.needs_data() && next_chunk_id < total_chunks
            };
            
            if needs_data {
                match fastn_p2p::client::call(
                    fetch_private_key.clone(),
                    fetch_target,
                    MediaProtocolV2::AudioStreamV2,
                    StreamRequest::RequestChunk { chunk_id: next_chunk_id },
                ).await {
                    Ok(StreamResponse::AudioChunk { chunk_id, data, is_last }) => {
                        {
                            let mut buffer_guard = fetch_buffer.lock().await;
                            buffer_guard.add_chunk(data);
                        }
                        println!("📥 Received chunk {} ({:.1}s buffered)", 
                                chunk_id, 
                                fetch_buffer.lock().await.current_buffer_ms as f64 / 1000.0);
                        next_chunk_id += 1;
                        
                        if is_last {
                            break;
                        }
                    }
                    Ok(StreamResponse::EndOfStream) => break,
                    Err(e) => {
                        eprintln!("❌ Failed to fetch chunk {}: {}", next_chunk_id, e);
                        break;
                    }
                    _ => {
                        eprintln!("❌ Unexpected response for chunk {}", next_chunk_id);
                        break;
                    }
                }
            } else {
                // Buffer is full or paused, wait a bit
                sleep(Duration::from_millis(100)).await;
            }
        }
        println!("📡 Chunk fetcher finished");
    });

    // Start audio player task
    let play_buffer = buffer.clone();
    let sink = Arc::new(sink);
    let play_sink = sink.clone();
    tokio::spawn(async move {
        loop {
            let chunk_data = {
                let mut buffer_guard = play_buffer.lock().await;
                if buffer_guard.playing {
                    buffer_guard.get_chunk()
                } else {
                    None
                }
            };
            
            if let Some(data) = chunk_data {
                // Convert to audio source and play
                let mut samples = Vec::with_capacity(data.len() / 2);
                for chunk_bytes in data.chunks_exact(2) {
                    let sample = i16::from_le_bytes([chunk_bytes[0], chunk_bytes[1]]);
                    samples.push(sample);
                }
                
                let source = rodio::buffer::SamplesBuffer::new(channels, sample_rate, samples);
                play_sink.append(source);
            } else {
                // No data available, wait
                sleep(Duration::from_millis(50)).await;
            }
        }
    });

    // Interactive controls
    tokio::spawn(async move {
        use std::io::Read;
        use termion::raw::IntoRawMode;
        
        let _raw = std::io::stdout().into_raw_mode().expect("Failed to enter raw mode");
        let mut stdin = std::io::stdin();
        let mut buffer_byte = [0u8; 1];
        
        loop {
            if stdin.read_exact(&mut buffer_byte).is_err() {
                break;
            }
            
            match buffer_byte[0] {
                b' ' => {
                    let mut buffer_guard = buffer.lock().await;
                    if buffer_guard.playing {
                        buffer_guard.pause();
                        sink.pause();
                        println!("\r⏸️  Paused            ");
                    } else {
                        buffer_guard.resume();
                        sink.play();
                        println!("\r▶️  Resumed           ");
                    }
                }
                b'q' | 27 => { // q or ESC
                    println!("\r⏹️  Stopping...       ");
                    break;
                }
                _ => {}
            }
        }
    });

    // Wait for playback to complete
    println!("🎼 Streaming started...");
    loop {
        sleep(Duration::from_millis(500)).await;
        
        let (buffer_ms, chunks_buffered) = {
            let buffer_guard = buffer.lock().await;
            (buffer_guard.current_buffer_ms, buffer_guard.chunks.len())
        };
        
        if buffer_ms == 0 && chunks_buffered == 0 && !sink.empty() {
            // Buffer empty and sink empty - stream finished
            break;
        }
    }

    println!("\n✅ Streaming completed!");
    Ok(())
}

async fn handle_audio_request(
    request: StreamRequest,
    state: AudioServerState,
) -> Result<StreamResponse, Box<dyn std::error::Error>> {
    let total_chunks = (state.audio_data.len() + state.chunk_size - 1) / state.chunk_size;
    
    match request {
        StreamRequest::GetStreamInfo => {
            println!("📊 Sending stream info to client");
            Ok(StreamResponse::StreamInfo {
                total_chunks: total_chunks as u64,
                chunk_size: state.chunk_size,
                sample_rate: state.sample_rate,
                channels: state.channels,
                duration_seconds: state.duration,
            })
        }
        StreamRequest::RequestChunk { chunk_id } => {
            if chunk_id >= total_chunks as u64 {
                return Ok(StreamResponse::EndOfStream);
            }
            
            let start_offset = (chunk_id as usize) * state.chunk_size;
            let end_offset = std::cmp::min(start_offset + state.chunk_size, state.audio_data.len());
            let chunk_data = state.audio_data[start_offset..end_offset].to_vec();
            let is_last = chunk_id == total_chunks as u64 - 1;
            
            println!("📦 Sending chunk {} ({} KB)", chunk_id, chunk_data.len() / 1024);
            
            Ok(StreamResponse::AudioChunk {
                chunk_id,
                data: chunk_data,
                is_last,
            })
        }
        StreamRequest::Stop => {
            println!("⏹️  Client requested stop");
            Ok(StreamResponse::EndOfStream)
        }
    }
}

// Audio decoding functions (copied from audio_test)
#[derive(Debug, thiserror::Error)]
pub enum MediaError {
    #[error("Audio file not found: {0}")]
    FileNotFound(String),
    #[error("Audio decode error: {0}")]
    DecodeError(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

async fn load_audio_file_with_format(filename: &str) -> Result<(Vec<u8>, u32, u16), MediaError> {
    let file_data = tokio::fs::read(filename).await
        .map_err(|_| MediaError::FileNotFound(filename.to_string()))?;
    
    // Use symphonia for OGG files
    use symphonia::core::audio::{AudioBufferRef, Signal};
    use symphonia::core::codecs::DecoderOptions;
    use symphonia::core::formats::FormatOptions;
    use symphonia::core::io::MediaSourceStream;
    use symphonia::core::meta::MetadataOptions;
    use symphonia::core::probe::Hint;
    
    let file_data_owned = file_data.to_vec();
    let cursor = std::io::Cursor::new(file_data_owned);
    let mss = MediaSourceStream::new(Box::new(cursor), Default::default());
    
    let hint = Hint::new();
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
    
    // Decode with proper stereo interleaving
    loop {
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(symphonia::core::errors::Error::IoError(e)) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                break;
            }
            Err(e) => {
                return Err(MediaError::DecodeError(format!("Packet read error: {:?}", e)));
            }
        };
        
        if packet.track_id() != track_id {
            continue;
        }
        
        match decoder.decode(&packet) {
            Ok(decoded) => {
                sample_rate = decoded.spec().rate;
                channels = decoded.spec().channels.count() as u16;
                
                // Proper stereo interleaving: [L,R,L,R,L,R...]
                match decoded {
                    AudioBufferRef::F32(buf) => {
                        let channels_count = buf.spec().channels.count();
                        let frames = buf.frames();
                        
                        for frame_idx in 0..frames {
                            for ch in 0..channels_count {
                                let sample = buf.chan(ch)[frame_idx];
                                let sample_i16 = (sample * 32767.0).clamp(-32767.0, 32767.0) as i16;
                                pcm_data.extend_from_slice(&sample_i16.to_le_bytes());
                            }
                        }
                    }
                    AudioBufferRef::S16(buf) => {
                        let channels_count = buf.spec().channels.count();
                        let frames = buf.frames();
                        
                        for frame_idx in 0..frames {
                            for ch in 0..channels_count {
                                let sample = buf.chan(ch)[frame_idx];
                                pcm_data.extend_from_slice(&sample.to_le_bytes());
                            }
                        }
                    }
                    _ => {
                        return Err(MediaError::DecodeError("Unsupported audio format".to_string()));
                    }
                }
            }
            Err(symphonia::core::errors::Error::IoError(_)) => break,
            Err(symphonia::core::errors::Error::DecodeError(_)) => continue,
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