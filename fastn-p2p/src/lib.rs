//! # fastn-p2p: High-Level Type-Safe P2P Communication
//!
//! This crate provides a high-level, type-safe API for P2P communication in the fastn ecosystem.
//! It builds on top of `fastn-net` but exposes only the essential, locked-down APIs that
//! reduce the possibility of bugs through strong typing and compile-time verification.
//!
//! ## Design Philosophy
//!
//! - **Type Safety First**: All communication uses strongly-typed REQUEST/RESPONSE/ERROR contracts
//! - **Minimal Surface Area**: Only essential APIs are exposed to reduce complexity
//! - **Bug Prevention**: API design makes common mistakes impossible or unlikely
//! - **Ergonomic**: High-level APIs handle boilerplate automatically
//!
//! ## Quick Start
//!
//! ### Request/Response Pattern
//!
//! ```rust,no_run
//! use fastn_p2p::SecretKey;
//! use serde::{Serialize, Deserialize};
//!
//! // Define your protocol
//! #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
//! pub enum EchoProtocol { Echo }
//!
//! #[derive(Serialize, Deserialize, Debug)]
//! pub struct EchoRequest { pub message: String }
//!
//! #[derive(Serialize, Deserialize, Debug)]
//! pub struct EchoResponse { pub echoed: String }
//!
//! #[derive(Serialize, Deserialize, Debug, thiserror::Error)]
//! #[error("Echo error: {message}")]
//! pub struct EchoError { pub message: String }
//!
//! type EchoResult = Result<EchoResponse, EchoError>;
//!
//! // Server: Handle requests
//! # async fn server_example() -> Result<(), Box<dyn std::error::Error>> {
//! let private_key = SecretKey::generate();
//! fastn_p2p::listen(private_key)
//!     .handle_requests(EchoProtocol::Echo, echo_handler)
//!     .await?;
//! # Ok(())
//! # }
//!
//! async fn echo_handler(req: EchoRequest) -> Result<EchoResponse, EchoError> {
//!     Ok(EchoResponse { echoed: format!("Echo: {}", req.message) })
//! }
//!
//! // Client: Make requests
//! # async fn client_example() -> Result<(), Box<dyn std::error::Error>> {
//! # let private_key = SecretKey::generate();
//! # let target_peer = SecretKey::generate().public_key();
//! let request = EchoRequest { message: "Hello P2P!".to_string() };
//! let result: EchoResult = fastn_p2p::client::call(
//!     private_key,
//!     target_peer,
//!     EchoProtocol::Echo,
//!     request
//! ).await?;
//!
//! match result {
//!     Ok(response) => println!("Response: {}", response.echoed),
//!     Err(error) => eprintln!("Error: {}", error),
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ### Streaming Pattern
//!
//! ```rust,no_run
//! use fastn_p2p::{SecretKey, Session};
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
//! pub enum FileProtocol { Download }
//!
//! // Server: Handle streams
//! # async fn server_example() -> Result<(), Box<dyn std::error::Error>> {
//! let private_key = SecretKey::generate();
//! fastn_p2p::listen(private_key)
//!     .handle_streams(FileProtocol::Download, (), file_handler)
//!     .await?;
//! # Ok(())
//! # }
//!
//! async fn file_handler(
//!     mut session: Session<FileProtocol>,
//!     filename: String,
//!     _state: (),
//! ) -> Result<(), std::io::Error> {
//!     let mut file = tokio::fs::File::open(&filename).await?;
//!     session.copy_from(&mut file).await?;
//!     Ok(())
//! }
//!
//! // Client: Receive stream
//! # async fn client_example() -> Result<(), Box<dyn std::error::Error>> {
//! # let private_key = SecretKey::generate();
//! # let target_peer = SecretKey::generate().public_key();
//! # let filename = "test.txt";
//! let mut session = fastn_p2p::client::connect(
//!     private_key,
//!     target_peer,
//!     FileProtocol::Download,
//!     filename,
//! ).await?;
//!
//! let mut output_file = tokio::fs::File::create("downloaded_file").await?;
//! session.copy_to(&mut output_file).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Core Concepts
//!
//! ### Protocols
//! Define your communication protocols as enums that implement `Serialize`, `Deserialize`, `Debug`, `Clone`, and `PartialEq`:
//!
//! ```rust
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
//! pub enum MyProtocol {
//!     RequestResponse,
//!     FileTransfer,
//!     ShellExecution,
//! }
//! ```
//!
//! ### Error Handling
//! Use structured error types with `thiserror` for application-level errors:
//!
//! ```rust
//! #[derive(Debug, thiserror::Error, serde::Serialize, serde::Deserialize)]
//! pub enum MyError {
//!     #[error("File not found: {0}")]
//!     NotFound(String),
//!     #[error("Permission denied: {0}")]
//!     PermissionDenied(String),
//! }
//! ```

extern crate self as fastn_p2p;

mod coordination;
mod globals;
mod handshake;
mod macros;

// Export server module (client is now separate fastn-p2p-client crate)
pub mod server;

// Export CLI module for magic CLI capabilities in serve_all
pub mod cli;

// Re-export modern server API for convenience
pub use server::{serve_all, echo_request_handler};

// Re-export serve_all types for convenience
pub use server::serve_all::BindingContext;

// Re-export essential types from fastn-net that users need
pub use fastn_net::{Graceful, Protocol};
// Note: PeerStreamSenders is intentionally NOT exported - users should use global singletons

// Re-export fastn-context::main for convenience
pub use fastn_context::main;

// Re-export key types for convenience
pub use fastn_id52::{PublicKey, SecretKey};

// Global singleton access - graceful is completely encapsulated in coordination module
pub use coordination::{cancelled, shutdown, spawn};
pub use globals::{graceful, pool};

// Server builder API - new clean interface
pub use server::builder_listen as listen;

// Legacy API exports (TODO: phase out in favor of builder API)
pub use server::{
    GetInputError, HandleRequestError, ListenerAlreadyActiveError, ListenerNotFoundError, Request,
    ResponseHandle, SendError, Session, active_listener_count, active_listeners, is_listening,
    listen as legacy_listen, stop_listening,
};
