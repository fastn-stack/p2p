//! Identity and Key Management
//! 
//! This module provides cryptographic identity management for P2P communication.
//! It was consolidated from the standalone fastn-id52 crate.

// Re-export the main types from the original fastn-id52 implementation
mod main;
pub use main::*;

// Re-export common types with cleaner names
pub use main::{SecretKey, PublicKey};