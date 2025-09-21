//! Type-safe protocol definitions for client-driven audio streaming
//! Uses string constants for protocol identification with clean request/response types.

// Protocol string constants - clean and efficient
pub const AUDIO_GET_INFO: &str = "audio.get_info";
pub const AUDIO_REQUEST_CHUNK: &str = "audio.request_chunk";  
pub const AUDIO_STOP: &str = "audio.stop";

// Get stream metadata
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct AudioInfoRequest;  // Empty struct for info request

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct AudioInfoResponse {
    pub total_chunks: u64,
    pub chunk_size_bytes: usize,
    pub chunk_duration_ms: u64,
    pub sample_rate: u32,
    pub channels: u16,
    pub total_duration_seconds: f64,
}

// Request audio chunk
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct AudioChunkRequest {
    pub chunk_id: u64,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct AudioChunkResponse {
    pub chunk_id: u64,
    pub data: Vec<u8>,
    pub is_last: bool,
}

// Stop streaming
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct AudioStopRequest;  // Empty struct for stop request

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct AudioStopResponse {
    pub stopped: bool,
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