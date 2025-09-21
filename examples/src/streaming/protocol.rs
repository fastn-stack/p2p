//! Protocol definitions for client-driven audio streaming

// Protocol enum for fastn-p2p
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub enum StreamingProtocol {
    AudioV2,
}

// Client requests to server
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum StreamRequest {
    /// Get basic stream information (duration, format, etc.)
    GetStreamInfo,
    /// Request a specific chunk of audio data
    RequestChunk { chunk_id: u64 },
    /// Stop streaming
    Stop,
}

// Server responses to client
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub enum StreamResponse {
    /// Stream metadata
    StreamInfo {
        total_chunks: u64,
        chunk_size_bytes: usize,
        chunk_duration_ms: u64,
        sample_rate: u32,
        channels: u16,
        total_duration_seconds: f64,
    },
    /// Audio chunk data
    AudioChunk {
        chunk_id: u64,
        data: Vec<u8>,
        is_last: bool,
    },
    /// End of stream
    EndOfStream,
    /// Error response
    Error(String),
}

// Client buffer status for adaptive streaming
#[derive(Debug)]
pub struct BufferStatus {
    pub buffered_chunks: usize,
    pub buffered_duration_ms: u64,
    pub target_buffer_ms: u64,
    pub is_playing: bool,
    pub needs_data: bool,
}