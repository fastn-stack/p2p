use serde::{Deserialize, Serialize};

/// Protocol for HTTP tunneling over P2P
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HttpProtocol {
    /// Forward HTTP requests/responses
    Forward,
}

/// HTTP request to be sent over P2P
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRequest {
    pub method: String,
    pub uri: String,
    pub headers: Vec<(String, String)>,
    // TODO: Change to streaming later
    pub body: Vec<u8>,
}

/// HTTP response to be sent over P2P
#[derive(Debug, Clone, Serialize, Deserialize)]  
pub struct HttpResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    // TODO: Change to streaming later
    pub body: Vec<u8>,
}

/// Errors that can occur during HTTP forwarding
#[derive(Debug, thiserror::Error)]
pub enum HttpError {
    #[error("Failed to connect to upstream: {0}")]
    UpstreamConnection(String),
    
    #[error("Failed to parse HTTP: {0}")]
    ParseError(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("P2P error: {source}")]
    P2p { source: eyre::Error },
}