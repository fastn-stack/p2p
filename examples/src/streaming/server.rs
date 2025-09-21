//! Audio server for client-driven streaming

use super::protocol::*;
use std::time::Instant;

// Server state - audio data loaded once at startup
#[derive(Clone)]
pub struct AudioServer {
    pub audio_data: Vec<u8>,
    pub sample_rate: u32,
    pub channels: u16,
    pub duration_seconds: f64,
    pub chunk_size: usize,
}

impl AudioServer {
    pub async fn new(audio_file: &str) -> Result<Self, Box<dyn std::error::Error>> {
        println!("📁 Loading audio file: {}", audio_file);
        let decode_start = Instant::now();
        
        let (audio_data, sample_rate, channels) = examples::audio_decoder::decode_audio_file(audio_file).await
            .map_err(|e| format!("Failed to decode audio: {}", e))?;
        
        let duration_seconds = audio_data.len() as f64 / (sample_rate as f64 * channels as f64 * 2.0);
        let decode_time = decode_start.elapsed();
        
        println!("✅ Audio loaded (+{:.3}s): {:.1}s, {}Hz, {} ch", 
                 decode_time.as_secs_f64(), duration_seconds, sample_rate, channels);
        
        Ok(Self {
            audio_data,
            sample_rate,
            channels,
            duration_seconds,
            chunk_size: 262144, // 256KB chunks
        })
    }
    
    pub fn get_stream_info(&self) -> StreamResponse {
        let total_chunks = (self.audio_data.len() + self.chunk_size - 1) / self.chunk_size;
        let chunk_duration_ms = (self.chunk_size as f64 / (self.sample_rate as f64 * self.channels as f64 * 2.0) * 1000.0) as u64;
        
        StreamResponse::StreamInfo {
            total_chunks: total_chunks as u64,
            chunk_size_bytes: self.chunk_size,
            chunk_duration_ms,
            sample_rate: self.sample_rate,
            channels: self.channels,
            total_duration_seconds: self.duration_seconds,
        }
    }
    
    pub fn get_chunk(&self, chunk_id: u64) -> StreamResponse {
        let total_chunks = (self.audio_data.len() + self.chunk_size - 1) / self.chunk_size;
        
        if chunk_id >= total_chunks as u64 {
            return StreamResponse::EndOfStream;
        }
        
        let start_offset = (chunk_id as usize) * self.chunk_size;
        let end_offset = std::cmp::min(start_offset + self.chunk_size, self.audio_data.len());
        let chunk_data = self.audio_data[start_offset..end_offset].to_vec();
        let is_last = chunk_id == total_chunks as u64 - 1;
        
        StreamResponse::AudioChunk {
            chunk_id,
            data: chunk_data,
            is_last,
        }
    }
}

// Request handler for fastn-p2p
pub async fn handle_request(
    request: StreamRequest,
    server: AudioServer,
) -> Result<StreamResponse, Box<dyn std::error::Error>> {
    match request {
        StreamRequest::GetStreamInfo => {
            println!("📊 Client requested stream info");
            Ok(server.get_stream_info())
        }
        StreamRequest::RequestChunk { chunk_id } => {
            println!("📦 Client requested chunk {} ({} KB)", 
                     chunk_id, server.chunk_size / 1024);
            Ok(server.get_chunk(chunk_id))
        }
        StreamRequest::Stop => {
            println!("⏹️  Client requested stop");
            Ok(StreamResponse::EndOfStream)
        }
    }
}