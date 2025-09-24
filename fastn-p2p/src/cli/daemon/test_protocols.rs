//! Test protocols for end-to-end testing
//!
//! These protocols are only used for testing the daemon functionality.
//! Production protocols will be implemented in separate crates.

use serde::{Deserialize, Serialize};

/// Protocol identifiers as const strings
pub const ECHO_PROTOCOL: &str = "Echo";
pub const SHELL_PROTOCOL: &str = "Shell";

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

// TODO: Add Shell protocol for interactive testing