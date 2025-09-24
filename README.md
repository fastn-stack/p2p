# fastn-p2p

Type-safe P2P communication library for Rust.

## Overview

`fastn-p2p` provides high-level, type-safe APIs for building peer-to-peer applications in Rust. Built on solid cryptographic foundations with clean abstractions for request/response and streaming patterns.

## Quick Start

### Request/Response Pattern

```rust
// Server side
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub enum EchoProtocol { Echo }

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct EchoRequest { pub message: String }

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct EchoResponse { pub echoed: String }

// Start server
let private_key = fastn_p2p::SecretKey::generate();
fastn_p2p::listen(private_key)
    .handle_requests(EchoProtocol::Echo, echo_handler)
    .await?;

async fn echo_handler(req: EchoRequest) -> Result<EchoResponse, EchoError> {
    Ok(EchoResponse { echoed: format!("Echo: {}", req.message) })
}
```

```rust
// Client side
let request = EchoRequest { message: "Hello P2P!".to_string() };
let response: EchoResult = fastn_p2p::client::call(
    private_key, 
    target_peer, 
    EchoProtocol::Echo, 
    request
).await?;
```

### Streaming Pattern

```rust
// Server side - stream handler
fastn_p2p::listen(private_key)
    .handle_streams(FileProtocol::Download, (), file_handler)
    .await?;

async fn file_handler(
    mut session: fastn_p2p::Session<FileProtocol>,
    filename: String,
    _state: (),
) -> Result<(), FileError> {
    let mut file = tokio::fs::File::open(&filename).await?;
    session.copy_from(&mut file).await?;
    Ok(())
}
```

```rust
// Client side - receive stream
let mut session = fastn_p2p::client::connect(
    private_key,
    target_peer,
    FileProtocol::Download,
    filename,
).await?;

let mut output_file = tokio::fs::File::create(&local_filename).await?;
session.copy_to(&mut output_file).await?;
```

## Features

- **🔐 Cryptographic Identity** - Ed25519 keys with ID52 encoding
- **📡 Request/Response** - Type-safe RPC-style communication
- **🌊 Streaming** - Efficient data streaming with `copy_to`/`copy_from`
- **🛡️ Type Safety** - Protocol enums and structured data
- **⚡ High Performance** - Built on async I/O foundations
- **📝 Rich Examples** - Complete working examples for all patterns

## Installation

```bash
cargo add fastn-p2p
```

## Examples

```bash
# Request/response pattern
cargo run --bin request_response server
cargo run --bin request_response client <id52> "Hello"

# File transfer streaming
cargo run --bin file_transfer server
cargo run --bin file_transfer client <id52> test.txt

# Remote shell execution  
cargo run --bin shell_simple daemon
cargo run --bin shell_simple exec <id52> whoami

# See examples/ directory for more patterns
```

## API Patterns

### Protocol Definition

Define your P2P protocols as enums:

```rust
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub enum MyProtocol {
    RequestResponse,
    FileTransfer,
}
```

### Error Handling

Use structured error types with `thiserror`:

```rust
#[derive(Debug, thiserror::Error)]
pub enum MyError {
    #[error("File not found: {0}")]
    NotFound(String),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
}
```

### Server Patterns

```rust
// Request/response handler
fastn_p2p::listen(private_key)
    .handle_requests(MyProtocol::RequestResponse, my_handler)
    .await?;

// Streaming handler with state
fastn_p2p::listen(private_key)
    .handle_streams(MyProtocol::FileTransfer, my_state, stream_handler)
    .await?;
```

## Documentation

- **[Examples](./examples/)** - Working examples for all major patterns
- **[API Reference](https://docs.rs/fastn-p2p)** - Complete API documentation

## License

Licensed under the Universal Permissive License (UPL-1.0).