//! User interface for interactive audio streaming

use super::client::AudioClient;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::sleep;

// Interactive streaming UI with play/pause controls
pub struct StreamingUI {
    client: Arc<Mutex<AudioClient>>,
    sink: Arc<rodio::Sink>,
}

impl StreamingUI {
    pub async fn new(mut client: AudioClient) -> Result<Self, Box<dyn std::error::Error>> {
        // Setup audio playback system
        let (_stream, stream_handle) = rodio::OutputStream::try_default()
            .map_err(|e| format!("Failed to create audio output: {}", e))?;
        
        let sink = rodio::Sink::try_new(&stream_handle)
            .map_err(|e| format!("Failed to create audio sink: {}", e))?;
        
        println!("🔧 Audio system ready");
        println!("💡 Press SPACE to pause/resume, 'q' to quit");
        
        Ok(Self {
            client: Arc::new(Mutex::new(client)),
            sink: Arc::new(sink),
        })
    }
    
    pub async fn start_streaming(self) -> Result<(), Box<dyn std::error::Error>> {
        // Start chunk fetcher task
        let fetch_client = self.client.clone();
        tokio::spawn(async move {
            let mut next_chunk_id = 0u64;
            
            loop {
                let needs_data = {
                    let client_guard = fetch_client.lock().await;
                    client_guard.needs_data() && next_chunk_id < client_guard.total_chunks
                };
                
                if needs_data {
                    match fetch_client.lock().await.request_chunk(next_chunk_id).await {
                        Ok(Some(_)) => {
                            let status = fetch_client.lock().await.get_buffer_status();
                            println!("📥 Chunk {} buffered ({:.1}s buffered)", 
                                    next_chunk_id, 
                                    status.buffered_duration_ms as f64 / 1000.0);
                            next_chunk_id += 1;
                        }
                        Ok(None) => {
                            println!("📡 End of stream");
                            break;
                        }
                        Err(e) => {
                            eprintln!("❌ Failed to fetch chunk {}: {}", next_chunk_id, e);
                            break;
                        }
                    }
                } else {
                    // Buffer full or paused
                    sleep(Duration::from_millis(100)).await;
                }
            }
        });
        
        // Start audio player task
        let play_client = self.client.clone();
        let play_sink = self.sink.clone();
        tokio::spawn(async move {
            loop {
                let chunk_data = {
                    let mut client_guard = play_client.lock().await;
                    client_guard.get_audio_chunk()
                };
                
                if let Some(data) = chunk_data {
                    // Convert to audio source
                    let mut samples = Vec::with_capacity(data.len() / 2);
                    for chunk_bytes in data.chunks_exact(2) {
                        let sample = i16::from_le_bytes([chunk_bytes[0], chunk_bytes[1]]);
                        samples.push(sample);
                    }
                    
                    let (sample_rate, channels) = {
                        let client_guard = play_client.lock().await;
                        (client_guard.sample_rate, client_guard.channels)
                    };
                    
                    let source = rodio::buffer::SamplesBuffer::new(channels, sample_rate, samples);
                    play_sink.append(source);
                } else {
                    // No data, wait
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
            let mut buffer = [0u8; 1];
            
            loop {
                if stdin.read_exact(&mut buffer).is_err() {
                    break;
                }
                
                match buffer[0] {
                    b' ' => {
                        let mut client_guard = self.client.lock().await;
                        let status = client_guard.get_buffer_status();
                        
                        if status.is_playing {
                            client_guard.pause();
                            self.sink.pause();
                            println!("⏸️  Paused");
                        } else {
                            client_guard.resume();
                            self.sink.play();
                            println!("▶️  Resumed");
                        }
                    }
                    b'q' | 27 => { // q or ESC
                        println!("⏹️  Stopping...");
                        break;
                    }
                    _ => {}
                }
            }
        });
        
        // Main loop - just wait and show status
        loop {
            sleep(Duration::from_millis(1000)).await;
            
            let status = self.client.lock().await.get_buffer_status();
            if !status.is_playing && status.buffered_chunks == 0 && self.sink.empty() {
                break;
            }
        }
        
        println!("\n✅ Streaming completed!");
        Ok(())
    }
}