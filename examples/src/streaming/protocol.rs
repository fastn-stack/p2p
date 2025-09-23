//! Clean streaming protocol types

// Protocol enum for current fastn-p2p API
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub enum StreamingProtocol {
    GetStream,
    ReadTrackRange,
}

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

