# fastn-p2p

Revolutionary P2P communication with centralized daemon coordination for Rust.

## Overview

`fastn-p2p` provides a **daemon-based P2P architecture** that eliminates the security, performance, and resource management issues of traditional direct P2P approaches. Applications use lightweight clients that communicate via a secure daemon, providing **automatic connection pooling**, **centralized key management**, and **operational excellence**.

## Architecture

```
┌─────────────────┐    Unix Socket     ┌──────────────────┐    P2P Network    ┌─────────────────┐
│ Client Apps     │◄──────────────────►│ fastn-p2p daemon │◄─────────────────►│ Remote Peers    │
│ (lightweight)   │   Type-safe API    │ (secure coord)   │   Real P2P        │ (protocols)     │
└─────────────────┘                    └──────────────────┘                    └─────────────────┘
```

**Revolutionary Benefits:**
- **🔒 Security**: Secret keys isolated in daemon, clients never handle crypto
- **⚡ Performance**: Automatic connection pooling, no repeated handshakes  
- **📦 Lightweight**: Client libraries 72% smaller (7 vs 25 dependencies)
- **🛠️ Operational**: Rich identity management, online/offline control

## Quick Start

### 1. Start the Daemon

```bash
# Start the P2P daemon (manages all identities and connections)
fastn-p2p daemon
```

### 2. Identity Management

```bash
# Create an identity
fastn-p2p create-identity alice

# Add a protocol to the identity  
fastn-p2p add-protocol alice --protocol Echo --config '{"max_length": 1000}'

# Set identity online
fastn-p2p identity-online alice

# Check status
fastn-p2p status
```

### 3. Client Applications (Lightweight)

```rust
// Add to Cargo.toml: fastn-p2p-client = "0.1"
use fastn_p2p_client;

// Send request via daemon (no secret keys in client!)
let request = EchoRequest { message: "Hello P2P!".to_string() };
let result = fastn_p2p_client::call(
    "alice",              // From identity (daemon manages keys)
    target_peer,          // To peer (PublicKey)
    "Echo",               // Protocol name
    "default",            // Protocol instance
    request               // Request data
).await?;
```

### 4. Server Applications

```rust
// Add to Cargo.toml: fastn-p2p = "0.1"  
use fastn_p2p;

// Server uses full fastn-p2p crate with daemon utilities
let private_key = load_identity_key("alice").await?;
fastn_p2p::listen(private_key)
    .handle_requests("Echo", echo_handler)
    .await?;

async fn echo_handler(req: EchoRequest) -> Result<EchoResponse, EchoError> {
    Ok(EchoResponse { echoed: format!("Echo: {}", req.message) })
}
```

## Crates

### fastn-p2p-client (Lightweight)
For client applications that send requests via daemon:
```bash
cargo add fastn-p2p-client  # 7 dependencies, no crypto
```

### fastn-p2p (Full Stack)  
For server applications and daemon infrastructure:
```bash
cargo add fastn-p2p         # 25 dependencies, full P2P stack
```

## Daemon Management

### Identity Operations
```bash
# Create identities
fastn-p2p create-identity alice
fastn-p2p create-identity bob

# Configure protocols for identities
fastn-p2p add-protocol alice --protocol Mail --config '{"storage_dir": "/var/mail"}'
fastn-p2p add-protocol alice --protocol Chat --alias backup --config '{"max_msgs": 1000}'

# Control identity state
fastn-p2p identity-online alice    # Enable alice's protocols
fastn-p2p identity-offline alice   # Disable for maintenance

# Remove protocols
fastn-p2p remove-protocol alice --protocol Mail --alias backup
```

### Operational Commands
```bash
fastn-p2p status              # Rich status dashboard
fastn-p2p daemon              # Start daemon (foreground)
```

## Client API (fastn-p2p-client)

### Request/Response
```rust
use fastn_p2p_client;

let result = fastn_p2p_client::call(
    "alice",              // From identity (daemon manages keys)
    target_peer,          // To peer (PublicKey)  
    "Mail",               // Protocol name
    "primary",            // Protocol instance
    mail_request          // Request data
).await?;
```

### Streaming  
```rust
let mut session = fastn_p2p_client::connect(
    "alice",              // From identity
    target_peer,          // To peer
    "FileTransfer",       // Protocol  
    "default",            // Instance
    filename              // Initial data
).await?;

session.copy_to(&mut local_file).await?;
```

## Server API (fastn-p2p)

### Protocol Servers
```rust
use fastn_p2p;

// Server applications use full fastn-p2p crate
let identity_key = load_identity("alice").await?;
fastn_p2p::listen(identity_key)
    .handle_requests("Echo", echo_handler)
    .await?;

async fn echo_handler(req: EchoRequest) -> Result<EchoResponse, EchoError> {
    Ok(EchoResponse { echoed: format!("Echo: {}", req.message) })
}
```

## Features

- **🔒 Secure by Design** - Secret keys never leave daemon
- **⚡ Connection Pooling** - Automatic connection reuse
- **📦 Lightweight Clients** - 72% fewer dependencies than direct P2P
- **🛠️ Operational Excellence** - Rich identity and protocol management
- **🌐 Real P2P** - Uses iroh for actual peer-to-peer networking
- **🎯 Type Safety** - PublicKey types throughout, no string confusion

## Testing

### Local Development
```bash
# Dual-daemon testing on local machine
./scripts/cli/test-request-response.sh

# Local + Digital Ocean droplet testing
./scripts/cli/test-do-p2p.sh
```

### CI/CD
- **GitHub Actions**: Automated dual-droplet testing on real internet
- **Production Validation**: Tests P2P across Digital Ocean infrastructure
- **Complete Coverage**: End-to-end validation with real network latency

## Examples

```bash
# Dual-mode request/response testing
cargo run --bin request_response server alice    # Start Echo server
cargo run --bin request_response client <id52>   # Send Echo request
```

## Documentation

- **[Examples](./examples/)** - Complete dual-daemon testing examples
- **[Scripts](./scripts/)** - End-to-end testing with real infrastructure  
- **[API Reference](https://docs.rs/fastn-p2p)** - Complete API documentation

## License

Licensed under the Universal Permissive License (UPL-1.0).