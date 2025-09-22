//! Clean streaming protocol types

// Protocol constants for fastn-p2p
pub const GET_STREAM: &str = "stream.get";
pub const READ_TRACK_RANGE: &str = "stream.read_range";

// Get stream metadata
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct GetStreamRequest {
    pub stream_name: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct GetStreamResponse {
    pub name: String,
    pub tracks: std::collections::HashMap<String, TrackInfo>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct TrackInfo {
    pub name: String,
    pub size_bytes: u64,
}

// Read track range
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct ReadTrackRangeRequest {
    pub stream_name: String,
    pub track_name: String,
    pub start: u64,
    pub length: u64,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct ReadTrackRangeResponse {
    pub data: Vec<u8>, // Will be bytes::Bytes in future
}

