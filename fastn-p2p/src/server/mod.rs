//! Server-side P2P functionality
//!
//! This module provides high-level, type-safe APIs for implementing P2P servers.

pub mod builder;
pub mod handle;
pub mod listener;
pub mod management;
pub mod request;
pub mod session;
pub mod daemon;

// Public API exports - no use statements, direct qualification
pub use builder::{ServerBuilder, listen as builder_listen};
pub use handle::{ResponseHandle, SendError};
pub use listener::listen;
pub use management::{
    ListenerAlreadyActiveError, ListenerNotFoundError, active_listener_count, active_listeners,
    is_listening, stop_listening,
};
pub use request::{GetInputError, HandleRequestError, Request};
pub use session::Session;

// Generic server utilities for applications
pub use daemon::{
    IdentityConfig, ProtocolBinding, ServerConfig, 
    ensure_fastn_home, load_all_identities, run_generic_server, acquire_singleton_lock
};
