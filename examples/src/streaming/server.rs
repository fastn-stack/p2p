//! Server-side stream provider

use super::protocol::*;

/// Stream provider trait - app implements this
pub trait StreamProvider: Send + Sync {
    async fn resolve_stream(&self, stream_name: &str) -> Option<ServerStream>;
    async fn read_track_range(&self, stream_name: &str, track_name: &str, start: u64, length: u64) -> Result<Vec<u8>, Box<dyn std::error::Error>>;
}

/// Server-side stream metadata
#[derive(Debug, Clone)]
pub struct ServerStream {
    pub name: String,
    pub tracks: std::collections::HashMap<String, ServerTrack>,
}

/// Server-side track metadata  
#[derive(Debug, Clone)]
pub struct ServerTrack {
    pub name: String,
    pub size_bytes: u64,
}

impl ServerStream {
    pub fn new(name: String) -> Self {
        // TODO: Initialize with name and empty tracks HashMap
        todo!()
    }
    
    pub fn add_track(&mut self, name: String, size_bytes: u64) {
        // TODO: Insert ServerTrack into tracks HashMap
        todo!()
    }
}

/// Handle GET_STREAM protocol requests
pub async fn handle_get_stream(
    request: GetStreamRequest,
    provider: &dyn StreamProvider,
) -> Result<GetStreamResponse, Box<dyn std::error::Error>> {
    // TODO: Print "Client requested stream: {stream_name}"
    // TODO: Call provider.resolve_stream(request.stream_name)
    // TODO: Convert ServerStream to GetStreamResponse (map ServerTrack to TrackInfo)
    // TODO: Return response or error if stream not found
    todo!()
}

/// Handle READ_TRACK_RANGE protocol requests
pub async fn handle_read_track_range(
    request: ReadTrackRangeRequest,
    provider: &dyn StreamProvider,
) -> Result<ReadTrackRangeResponse, Box<dyn std::error::Error>> {
    // TODO: Print "Client reading {stream}.{track} range {start}..{start+length}"
    // TODO: Call provider.read_track_range(stream_name, track_name, start, length)
    // TODO: Return ReadTrackRangeResponse with data
    // TODO: Handle errors (stream/track not found, invalid range)
    todo!()
}