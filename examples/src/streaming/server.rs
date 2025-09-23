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
        Self {
            name,
            tracks: std::collections::HashMap::new(),
        }
    }
    
    pub fn add_track(&mut self, name: String, size_bytes: u64) {
        // TODO: Insert ServerTrack into tracks HashMap
        let track = ServerTrack { name: name.clone(), size_bytes };
        self.tracks.insert(name, track);
    }
}

/// Handle GET_STREAM protocol requests
pub async fn handle_get_stream<T: StreamProvider>(
    request: GetStreamRequest,
    provider: &T,
) -> Result<GetStreamResponse, Box<dyn std::error::Error>> {
    // TODO: Print "Client requested stream: {stream_name}"
    println!("📊 Client requested stream: {}", request.stream_name);
    
    match provider.resolve_stream(&request.stream_name).await {
        Some(server_stream) => {
            let tracks = server_stream.tracks.into_iter()
                .map(|(name, server_track)| (name.clone(), TrackInfo {
                    name,
                    size_bytes: server_track.size_bytes,
                }))
                .collect();
            
            Ok(GetStreamResponse {
                name: server_stream.name,
                tracks,
            })
        }
        None => Err(format!("Stream '{}' not found", request.stream_name).into())
    }
}

/// Handle READ_TRACK_RANGE protocol requests
pub async fn handle_read_track_range<T: StreamProvider>(
    request: ReadTrackRangeRequest,
    provider: &T,
) -> Result<ReadTrackRangeResponse, Box<dyn std::error::Error>> {
    // TODO: Print "Client reading {stream}.{track} range {start}..{start+length}"
    println!("📦 Reading {}.{} range {}..{}", 
             request.stream_name, request.track_name, 
             request.start, request.start + request.length);
    
    let data = provider.read_track_range(
        &request.stream_name,
        &request.track_name,
        request.start,
        request.length,
    ).await?;
    
    Ok(ReadTrackRangeResponse { data })
}