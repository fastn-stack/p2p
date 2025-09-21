//! Audio client for client-driven streaming

use super::protocol::*;
use std::collections::VecDeque;
use std::time::Instant;
use tokio::sync::mpsc;

// Client-side audio buffer manager
#[derive(Debug)]
pub struct AudioBuffer {
    chunks: VecDeque<Vec<u8>>,
    target_buffer_ms: u64,
    current_buffer_ms: u64,
    chunk_duration_ms: u64,
    is_playing: bool,
}

impl AudioBuffer {
    pub fn new(target_buffer_ms: u64, chunk_duration_ms: u64) -> Self {
        Self {
            chunks: VecDeque::new(),
            target_buffer_ms,
            current_buffer_ms: 0,
            chunk_duration_ms,
            is_playing: true,
        }
    }
    
    pub fn needs_data(&self) -> bool {
        self.is_playing && self.current_buffer_ms < self.target_buffer_ms
    }
    
    pub fn add_chunk(&mut self, data: Vec<u8>) {
        self.chunks.push_back(data);
        self.current_buffer_ms += self.chunk_duration_ms;
    }
    
    pub fn get_chunk(&mut self) -> Option<Vec<u8>> {
        if let Some(chunk) = self.chunks.pop_front() {
            self.current_buffer_ms = self.current_buffer_ms.saturating_sub(self.chunk_duration_ms);
            Some(chunk)
        } else {
            None
        }
    }
    
    pub fn pause(&mut self) {
        self.is_playing = false;
    }
    
    pub fn resume(&mut self) {
        self.is_playing = true;
    }
    
    pub fn status(&self) -> BufferStatus {
        BufferStatus {
            buffered_chunks: self.chunks.len(),
            buffered_duration_ms: self.current_buffer_ms,
            target_buffer_ms: self.target_buffer_ms,
            is_playing: self.is_playing,
            needs_data: self.needs_data(),
        }
    }
}

// Audio client for P2P streaming
pub struct AudioClient {
    private_key: fastn_p2p::SecretKey,
    target: fastn_p2p::PublicKey,
    buffer: AudioBuffer,
    // Stream info
    pub total_chunks: u64,
    pub sample_rate: u32,
    pub channels: u16,
    pub duration_seconds: f64,
}

impl AudioClient {
    pub async fn connect(target: fastn_p2p::PublicKey) -> Result<Self, Box<dyn std::error::Error>> {
        let private_key = fastn_p2p::SecretKey::generate();
        let connect_start = Instant::now();
        
        println!("🔍 Getting stream info...");
        
        // Get stream info
        let stream_info: StreamResponse = fastn_p2p::client::call(
            private_key.clone(),
            target,
            StreamingProtocol::AudioV2,
            StreamRequest::GetStreamInfo,
        ).await?;
        
        let (total_chunks, chunk_duration_ms, sample_rate, channels, duration_seconds) = match stream_info {
            StreamResponse::StreamInfo { 
                total_chunks, 
                chunk_duration_ms, 
                sample_rate, 
                channels, 
                total_duration_seconds,
                ..
            } => {
                println!("✅ Stream info received (+{:.3}s)", connect_start.elapsed().as_secs_f64());
                println!("📊 Stream: {:.1}s, {}Hz, {} ch, {} chunks", 
                         total_duration_seconds, sample_rate, channels, total_chunks);
                (total_chunks, chunk_duration_ms, sample_rate, channels, total_duration_seconds)
            }
            _ => return Err("Failed to get stream info".into()),
        };
        
        // Create buffer with 3 second target
        let target_buffer_ms = 3000;
        let buffer = AudioBuffer::new(target_buffer_ms, chunk_duration_ms);
        
        println!("🔊 Buffer target: {:.1}s ({} chunks)", 
                 target_buffer_ms as f64 / 1000.0,
                 target_buffer_ms / chunk_duration_ms);
        
        Ok(Self {
            private_key,
            target,
            buffer,
            total_chunks,
            sample_rate,
            channels,
            duration_seconds,
        })
    }
    
    pub async fn request_chunk(&mut self, chunk_id: u64) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error>> {
        let response: StreamResponse = fastn_p2p::client::call(
            self.private_key.clone(),
            self.target,
            StreamingProtocol::AudioV2,
            StreamRequest::RequestChunk { chunk_id },
        ).await?;
        
        match response {
            StreamResponse::AudioChunk { data, is_last, .. } => {
                self.buffer.add_chunk(data.clone());
                Ok(Some(data))
            }
            StreamResponse::EndOfStream => Ok(None),
            StreamResponse::Error(e) => Err(e.into()),
            _ => Err("Unexpected response".into()),
        }
    }
    
    pub fn get_buffer_status(&self) -> BufferStatus {
        self.buffer.status()
    }
    
    pub fn get_audio_chunk(&mut self) -> Option<Vec<u8>> {
        self.buffer.get_chunk()
    }
    
    pub fn pause(&mut self) {
        self.buffer.pause();
    }
    
    pub fn resume(&mut self) {
        self.buffer.resume();
    }
    
    pub fn needs_data(&self) -> bool {
        self.buffer.needs_data()
    }
}