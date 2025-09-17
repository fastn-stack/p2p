//! # fastn-p2p
//!
//! Type-safe P2P communication library for Rust.
//!
//! ## Quick Start
//!
//! ```rust
//! use fastn_p2p::{SecretKey, PublicKey};
//!
//! // Generate peer identity
//! let secret_key = SecretKey::generate();
//! let peer_id = secret_key.public_key().to_string();  // 52-char ID52
//!
//! // Sign and verify messages
//! let message = b"Hello, P2P!";
//! let signature = secret_key.sign(message);
//! assert!(secret_key.public_key().verify(message, &signature).is_ok());
//! ```
//!
//! ## Documentation
//!
//! - [Identity & Keys](https://github.com/fastn-stack/p2p/blob/main/docs/identity.md) - Detailed key management guide
//! - [Examples](https://github.com/fastn-stack/p2p/tree/main/examples) - Working example applications
//!
//! ## CLI Tool
//!
//! ```bash
//! # Install the key generation tool
//! cargo install fastn-p2p
//!
//! # Generate a new peer identity
//! fastn-p2p-keygen generate
//! ```

mod errors;
mod keyring;
mod keys;

pub use errors::{
    InvalidKeyBytesError, InvalidSignatureBytesError, ParseId52Error, ParseSecretKeyError,
    SignatureVerificationError,
};
pub use keyring::KeyringError;
pub use keys::{PublicKey, SecretKey, Signature};

#[cfg(feature = "dns")]
pub use errors::ResolveError;

#[cfg(feature = "dns")]
pub mod dns;