//! Protocol definitions for client-driven audio streaming

// Protocol enum for current fastn-p2p API (will migrate to &'static str later - see GitHub issue #2)
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub enum AudioProtocol {
    GetInfo,
    RequestChunk,
}

// Audio channel configuration
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Copy)]
pub enum AudioChannels {
    Mono = 1,
    Stereo = 2,
}

// Get stream metadata request/response
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct GetInfoRequest {
    /// Preferred chunk size in bytes (server may adjust based on capabilities)
    pub preferred_chunk_size_bytes: Option<usize>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct GetInfoResponse {
    pub total_chunks: u64,
    pub chunk_size_bytes: usize,
    pub chunk_duration_ms: u64,
    pub sample_rate: u32,
    pub channels: AudioChannels,
    pub total_duration_ms: u64,
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

