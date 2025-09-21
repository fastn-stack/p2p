//! Protocol definitions for client-driven audio streaming

// Protocol enum for current fastn-p2p API (will migrate to &'static str later - see GitHub issue #2)
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub enum AudioProtocol {
    GetInfo,
    RequestChunk,
}

// Get stream metadata request/response
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct GetInfoRequest;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct GetInfoResponse {
    pub total_chunks: u64,
    pub chunk_size_bytes: usize,
    pub chunk_duration_ms: u64,
    pub sample_rate: u32,
    pub channels: u16,
    pub total_duration_seconds: f64,
}

// Request audio chunk request/response
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct RequestChunkRequest {
    pub chunk_id: u64,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct RequestChunkResponse {
    pub chunk_id: u64,
    pub data: Vec<u8>,
    pub is_last: bool,
}

// Client buffer status for adaptive streaming
#[derive(Debug, Clone)]
pub struct BufferStatus {
    pub buffered_chunks: usize,
    pub buffered_duration_ms: u64,
    pub target_buffer_ms: u64,
    pub is_playing: bool,
    pub needs_data: bool,
}