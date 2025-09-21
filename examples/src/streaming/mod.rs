//! Client-driven audio streaming module
//!
//! Clean separation of concerns:
//! - protocol: Message types and protocol definitions
//! - server: Audio server that responds to chunk requests
//! - client: Audio client with buffer management  
//! - ui: Interactive controls and user interface

pub mod protocol;
pub mod server;
pub mod client;
pub mod ui;

// Re-export key types for convenience
pub use protocol::*;
pub use server::AudioServer;
pub use client::AudioClient;
pub use ui::StreamingUI;