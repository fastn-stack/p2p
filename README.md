# fastn-p2p

Type-safe P2P communication library for Rust.

## Overview

`fastn-p2p` provides high-level, type-safe APIs for building peer-to-peer applications in Rust. Built on top of [iroh](https://iroh.computer/), it offers clean abstractions for common P2P patterns.

## Features

- **Type-safe protocols** - Compile-time verification of message types
- **Simple client/server patterns** - Easy request/response communication  
- **Identity management** - Secure key generation and management
- **Async/await support** - Modern Rust async patterns
- **Minimal setup** - Get started with just a few lines of code

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
fastn-p2p = "0.1"
```

### Echo Server Example

```rust
use fastn_p2p::{SecretKey, serve};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
enum Protocol { Echo }

#[fastn_p2p::main]
async fn main() -> eyre::Result<()> {
    let key = SecretKey::generate();
    println!("Server ID: {}", key.public_key().id52());
    
    let protocols = [Protocol::Echo];
    let mut stream = fastn_p2p::listen!(key, &protocols);
    
    while let Some(request) = stream.next().await {
        request.handle(|msg: String| async move {
            Ok::<String, String>(format!("Echo: {}", msg))
        }).await?;
    }
    
    Ok(())
}
```

### Client Example

```rust
use fastn_p2p::{SecretKey, client};

#[fastn_p2p::main] 
async fn main() -> eyre::Result<()> {
    let key = SecretKey::generate();
    let target = "12D3KooW...".parse()?; // Server's ID
    
    let response: Result<String, String> = fastn_p2p::client::call(
        key, target, Protocol::Echo, "Hello P2P!".to_string()
    ).await?;
    
    println!("Response: {:?}", response);
    Ok(())
}
```

## Examples

See the [`examples/`](./examples/) directory for complete working examples:

- **Echo**: Basic request/response communication
- **Remote Shell**: Execute commands on remote machines
- More examples coming soon!

## Architecture

This crate consolidates several components:
- **Identity management** (from fastn-id52)
- **Network utilities** (from fastn-net)  
- **High-level P2P APIs** (fastn-p2p)

## License

Licensed under the Universal Permissive License (UPL-1.0).

## Contributing

Contributions welcome! Please open an issue to discuss major changes.