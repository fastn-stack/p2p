//! Network Utilities
//! 
//! This module provides low-level networking utilities for P2P communication.
//! It was consolidated from the standalone fastn-net crate.

// Re-export all the networking functionality
mod graceful;
mod lib;
mod peer_stream_senders;

pub use graceful::*;
pub use lib::*;
pub use peer_stream_senders::*;