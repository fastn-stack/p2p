//! Client-side streaming types

use super::protocol::*;

/// Client-side stream - no connection info embedded
#[derive(Debug)]
pub struct ClientStream {
    pub name: String,
    pub tracks: std::collections::HashMap<String, ClientTrack>,
}

/// Client-side track - clean without connection details
#[derive(Debug)]
pub struct ClientTrack {
    pub name: String,
    pub size_bytes: u64,
}

impl ClientStream {
    /// Create client stream from server response
    pub fn from_response(response: GetStreamResponse) -> Self {
        // TODO: Convert GetStreamResponse to ClientStream
        // TODO: Map TrackInfo to ClientTrack
        let tracks = response.tracks.into_iter()
            .map(|(name, track_info)| (name.clone(), ClientTrack {
                name,
                size_bytes: track_info.size_bytes,
            }))
            .collect();
        
        Self {
            name: response.name,
            tracks,
        }
    }
    
    pub fn get_track(&self, track_name: &str) -> Option<&ClientTrack> {
        // TODO: Return track from HashMap or None
        self.tracks.get(track_name)
    }
    
    pub fn list_tracks(&self) -> Vec<String> {
        // TODO: Return Vec of track names from HashMap keys
        self.tracks.keys().cloned().collect()
    }
}

/// Stream client - handles P2P communication separately from stream data
pub struct StreamClient {
    private_key: fastn_p2p::SecretKey,
    server_id: fastn_p2p::PublicKey,
}

impl StreamClient {
    pub fn new(server_id: fastn_p2p::PublicKey) -> Self {
        // TODO: Generate private_key = fastn_p2p::SecretKey::generate()
        Self {
            private_key: fastn_p2p::SecretKey::generate(),
            server_id,
        }
    }
    
    /// Open stream by name
    pub async fn open_stream(&self, stream_name: &str) -> Result<ClientStream, Box<dyn std::error::Error>> {
        // TODO: Call fastn_p2p::client::call() with GET_STREAM protocol
        // TODO: Send GetStreamRequest with stream_name
        // TODO: Parse GetStreamResponse
        // TODO: Convert to ClientStream using ClientStream::from_response()
        // TODO: Return ClientStream
        todo!()
    }
    
    /// Read range from specific track
    pub async fn read_track_range(
        &self, 
        stream_name: &str, 
        track_name: &str, 
        start: u64, 
        length: u64
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // TODO: Call fastn_p2p::client::call() with READ_TRACK_RANGE protocol
        // TODO: Send ReadTrackRangeRequest with stream_name, track_name, start, length
        // TODO: Parse ReadTrackRangeResponse  
        // TODO: Return response.data
        // TODO: Handle errors (track not found, invalid range)
        todo!()
    }
}