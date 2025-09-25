//! # fastn-p2p-client: Lightweight P2P Client Library
//!
//! This crate provides a lightweight client library for fastn P2P communication
//! that works via the fastn-p2p daemon. It has minimal dependencies and provides
//! the same API as the examples, but routes through the daemon instead of direct P2P.
//!
//! ## Architecture
//!
//! ```
//! ┌─────────────────┐    Unix Socket     ┌──────────────────┐
//! │ fastn-p2p-client│◄──────────────────►│ fastn-p2p daemon │
//! │  (lightweight)  │                    │    (full stack)  │
//! └─────────────────┘                    └──────────────────┘
//! ```
//!
//! ## Usage
//!
//! The API matches the original examples but routes through the daemon:
//!
//! ```rust,no_run
//! // Same API as examples, but daemon-powered
//! use fastn_p2p_client as fastn_p2p;
//!
//! let result = fastn_p2p::client::call(
//!     private_key, target_peer, protocol, request
//! ).await?;
//! ```

pub mod client;
pub mod error;

// Re-export only PublicKey for peer identification (no SecretKey - daemon manages all keys)
pub use fastn_id52::PublicKey;

// Re-export client functions for convenience  
pub use client::{call, connect, Session};

/// Error type for client operations
pub use error::{ClientError, ConnectionError};

// Re-export main macro for examples compatibility
pub use fastn_context::main;