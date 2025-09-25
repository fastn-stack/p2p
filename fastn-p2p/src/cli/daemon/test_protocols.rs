//! Test protocols for end-to-end testing
//!
//! These protocols are only used for testing the daemon functionality.
//! Production protocols will be implemented in separate crates.

use serde::{Deserialize, Serialize};

/// Protocol identifiers as const strings
pub const ECHO_PROTOCOL: &str = "Echo";
pub const SHELL_PROTOCOL: &str = "Shell";

/// Echo Protocol Types (moved from request_response example)
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub enum EchoProtocol {
    Echo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EchoRequest {
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EchoResponse {
    pub echoed: String,
}

#[derive(Debug, Serialize, Deserialize, thiserror::Error)]
pub enum EchoError {
    #[error("Invalid message: {0}")]
    InvalidMessage(String),
}

pub type EchoResult = Result<EchoResponse, EchoError>;

/// Echo request handler (moved from request_response example)
pub async fn echo_handler(req: EchoRequest) -> Result<EchoResponse, EchoError> {
    println!("💬 Received: {}", req.message);
    
    // Basic validation  
    if req.message.is_empty() {
        return Err(EchoError::InvalidMessage("Message cannot be empty".to_string()));
    }
    
    Ok(EchoResponse {
        echoed: format!("Echo: {}", req.message),
    })
}

// TODO: Add Shell protocol for interactive testing