//! Audio server for client-driven streaming

use super::protocol::*;
use std::time::Instant;

/// Audio server state - holds decoded audio data and metadata
#[derive(Clone)]
pub struct AudioServer {
    pub audio_data: Vec<u8>,
    pub sample_rate: u32,
    pub channels: u16,
    pub duration_seconds: f64,
    pub chunk_size: usize,
}

impl AudioServer {
    /// Create new audio server by loading and decoding audio file
    pub async fn new(audio_file: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // TODO: Load audio file using examples::audio_decoder::decode_audio_file()
        // TODO: Calculate duration_seconds from PCM data length
        // TODO: Set chunk_size to 256KB (262144 bytes)
        // TODO: Print timing info and audio metadata
        // TODO: Return AudioServer instance
        todo!()
    }
    
    /// Get stream information for client
    pub fn get_stream_info(&self) -> GetInfoResponse {
        // TODO: Calculate total_chunks = (audio_data.len() + chunk_size - 1) / chunk_size
        // TODO: Calculate chunk_duration_ms based on sample_rate, channels, chunk_size
        // TODO: Return GetInfoResponse with all metadata
        todo!()
    }
    
    /// Get specific audio chunk by ID
    pub fn get_chunk(&self, chunk_id: u64) -> Option<RequestChunkResponse> {
        // TODO: Check if chunk_id is valid (< total_chunks)
        // TODO: Calculate start_offset = chunk_id * chunk_size
        // TODO: Calculate end_offset = min(start_offset + chunk_size, audio_data.len())
        // TODO: Extract chunk_data = audio_data[start_offset..end_offset]
        // TODO: Check if is_last = (chunk_id == total_chunks - 1)
        // TODO: Return RequestChunkResponse with chunk_id, data, is_last
        todo!()
    }
}

/// Handle GetInfo protocol requests
pub async fn handle_get_info(
    _request: GetInfoRequest,
    server: AudioServer,
) -> Result<GetInfoResponse, Box<dyn std::error::Error>> {
    // TODO: Print "Client requested stream info"
    // TODO: Call server.get_stream_info()
    // TODO: Return the response
    todo!()
}

/// Handle RequestChunk protocol requests  
pub async fn handle_request_chunk(
    request: RequestChunkRequest,
    server: AudioServer,
) -> Result<RequestChunkResponse, Box<dyn std::error::Error>> {
    // TODO: Print "Client requested chunk {chunk_id} ({size} KB)"
    // TODO: Call server.get_chunk(request.chunk_id)
    // TODO: Handle None case (chunk not found) - return error
    // TODO: Return the chunk response
    todo!()
}