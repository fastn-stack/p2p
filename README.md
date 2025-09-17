# fastn-p2p

Type-safe P2P communication library for Rust.

## Overview

`fastn-p2p` provides high-level, type-safe APIs for building peer-to-peer applications in Rust. Built on solid cryptographic foundations with clean abstractions for common P2P patterns.

## Quick Start

```rust
use fastn_p2p::{SecretKey, PublicKey};

// Generate peer identity
let secret_key = SecretKey::generate();
let peer_id = secret_key.public_key().to_string();  // 52-char ID52

println!("Peer ID: {}", peer_id);
```

## Features

- **🔐 Cryptographic Identity** - Ed25519 keys with ID52 encoding
- **🔧 Key Management** - Secure storage in system keyring
- **📡 P2P Communication** - High-level APIs for peer communication *(coming soon)*
- **📝 Examples** - Reference implementations and usage patterns

## Installation

```bash
cargo add fastn-p2p
```

## Examples

```bash
# Generate keys
cargo run --example keygen

# More examples coming as P2P features are added
```

## Documentation

- **[Identity & Keys](./docs/identity.md)** - Cryptographic identity management
- **[API Reference](https://docs.rs/fastn-p2p)** - Complete API documentation

## Status

🚧 **Under Development** - Core identity management is stable. P2P communication APIs are being added incrementally.

## Roadmap

- **✅ Phase 1**: Cryptographic identity and key management
- **🚧 Phase 2**: P2P communication patterns  
- **📋 Phase 3**: High-level application APIs
- **📋 Phase 4**: Example applications

## License

Licensed under the Universal Permissive License (UPL-1.0).