//! Protocol multiplexing over P2P connections.
//!
//! This module implements a custom protocol multiplexing system over Iroh P2P
//! connections, deliberately deviating from Iroh's recommended ALPN-per-protocol
//! approach.
//!
//! # Why Not Use Iroh's Built-in ALPN Feature?
//!
//! Iroh [recommends using different ALPNs](https://docs.rs/iroh/latest/iroh/endpoint/struct.Builder.html#method.alpns)
//! for different protocols. However, this approach has a significant limitation:
//! **each protocol requires a separate connection**.
//!
//! ## The Problem with Multiple Connections
//!
//! Consider a typical P2P session where an entity might:
//! - Send periodic pings to check connection health
//! - Proxy HTTP requests through another entity
//! - Tunnel TCP connections simultaneously
//! - Stream real-time data (e.g., during a call while browsing shared files)
//!
//! With Iroh's approach, each protocol would need its own connection, requiring
//! a full TLS handshake for each. ALPN is negotiated during the TLS handshake:
//!
//! ```text
//! Client Hello Message Structure:
//! ┌─────────────────────────────────────┐
//! │ Handshake Type: Client Hello (1)    │
//! │ Version: TLS 1.2 (0x0303)          │
//! │ Random: dd67b5943e5efd07...        │
//! │ Cipher Suites: [...]                │
//! │ Extensions:                         │
//! │   ALPN Extension:                   │
//! │     - h2                           │
//! │     - http/1.1                     │
//! └─────────────────────────────────────┘
//! ```
//!
//! Creating additional connections means additional:
//! - TLS handshakes (expensive cryptographic operations)
//! - Network round trips
//! - Memory overhead for connection state
//! - Complexity in connection management
//!
//! ## Our Solution: Application-Layer Multiplexing
//!
//! We use a single ALPN (`/fastn/entity/0.1`) and multiplex different protocols
//! over [bidirectional streams](https://docs.rs/iroh/latest/iroh/endpoint/struct.Connection.html#method.open_bi)
//! within that connection:
//!
//! ```text
//! Single Connection between Entities
//!     ├── Stream 1: HTTP Proxy
//!     ├── Stream 2: Ping
//!     ├── Stream 3: TCP Tunnel
//!     └── Stream N: ...
//! ```
//!
//! Each stream starts with a JSON protocol header identifying its type.
//!
//! # The Protocol "Protocol"
//!
//! ## Stream Lifecycle
//!
//! 1. **Client entity** opens a bidirectional stream
//! 2. **Client** sends a JSON protocol header (newline-terminated)
//! 3. **Server entity** sends ACK to confirm protocol support
//! 4. Protocol-specific communication begins
//!
//! ## Protocol Header
//!
//! The first message on each stream is a JSON-encoded [`ProtocolHeader`] containing:
//! - The [`Protocol`] type (Ping, Http, Tcp, etc.)
//! - Optional protocol-specific metadata
//!
//! This allows protocol handlers to receive all necessary information upfront
//! without additional negotiation rounds.
//!
//! # Future Considerations
//!
//! This multiplexing approach may not be optimal for all use cases. Real-time
//! protocols (RTP/RTCP for audio/video) might benefit from dedicated connections
//! to avoid head-of-line blocking. This design decision will be re-evaluated
//! based on performance requirements.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub enum Protocol {
    /// Active built-in protocols
    Ping,
    WhatTimeIsIt,
    Http,
    HttpProxy,
    Socks5,
    Tcp,
    
    /// Revolutionary per-application protocols for serve_all() architecture
    /// Protocol names like "mail.fastn.com", "echo.fastn.com", etc.
    Application(String),
    
    /// Legacy generic protocol (still used by existing fastn-p2p code)
    Generic(serde_json::Value),
}

impl std::fmt::Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Protocol::Ping => write!(f, "Ping"),
            Protocol::WhatTimeIsIt => write!(f, "WhatTimeIsIt"),
            Protocol::Http => write!(f, "Http"),
            Protocol::HttpProxy => write!(f, "HttpProxy"),
            Protocol::Socks5 => write!(f, "Socks5"),
            Protocol::Tcp => write!(f, "Tcp"),
            Protocol::Application(name) => write!(f, "{}", name),
            Protocol::Generic(value) => write!(f, "Generic({value})"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_generic() {
        // Test Generic variant serialization
        let generic_value = serde_json::json!({"type": "custom", "version": 1});
        let protocol = Protocol::Generic(generic_value.clone());

        let serialized = serde_json::to_string(&protocol).unwrap();
        let deserialized: Protocol = serde_json::from_str(&serialized).unwrap();

        match deserialized {
            Protocol::Generic(value) => assert_eq!(value, generic_value),
            _ => panic!("Expected Generic variant"),
        }
    }
}

/// Single ALPN protocol identifier for all fastn entity connections.
///
/// Each fastn instance is called an "entity" in the P2P network. Unlike Iroh's
/// recommended approach of using different ALPNs for different protocols, we use
/// a single ALPN and multiplex protocols at the application layer. This avoids
/// the overhead of multiple TLS handshakes when entities need to use multiple
/// protocols (e.g., HTTP proxy + TCP tunnel + ping).
///
/// See module documentation for detailed rationale.
pub const APNS_IDENTITY: &[u8] = b"/fastn/entity/0.1";

/// Protocol header for both built-in and revolutionary serve_all() protocols.
///
/// Sent at the beginning of each bidirectional stream to identify
/// the protocol and provide routing information when needed.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ProtocolHeader {
    /// Protocol identifier
    pub protocol: Protocol,
    
    /// Command within the protocol (for Application protocols only)
    pub command: Option<String>,
    
    /// Protocol binding alias (for Application protocols only)  
    pub bind_alias: Option<String>,
    
    /// CLI arguments support (issue #13: stdargs)
    pub args: Vec<String>,
    
    /// Legacy compatibility for extra protocol data
    pub extra: Option<String>,
}

impl From<Protocol> for ProtocolHeader {
    fn from(protocol: Protocol) -> Self {
        Self {
            protocol,
            command: None,       // Built-in protocols don't need commands
            bind_alias: None,    // Built-in protocols don't need bind aliases
            args: Vec::new(),
            extra: None,
        }
    }
}

impl ProtocolHeader {
    /// Create protocol header for serve_all() Application protocols
    pub fn for_application(
        protocol_name: String,
        command: String,
        bind_alias: String,
        args: Vec<String>,
    ) -> Self {
        Self {
            protocol: Protocol::Application(protocol_name),
            command: Some(command),
            bind_alias: Some(bind_alias),
            args,
            extra: None,
        }
    }
    
    /// Get routing information for serve_all() (returns None for built-in protocols)
    pub fn serve_all_routing(&self) -> Option<(&str, &str, &str, &[String])> {
        match &self.protocol {
            Protocol::Application(protocol_name) => {
                if let (Some(command), Some(bind_alias)) = (&self.command, &self.bind_alias) {
                    Some((protocol_name, command, bind_alias, &self.args))
                } else {
                    None
                }
            }
            _ => None,  // Built-in protocols don't have serve_all routing
        }
    }
    
    /// Check if this is a built-in protocol
    pub fn is_builtin(&self) -> bool {
        !matches!(self.protocol, Protocol::Application(_))
    }
}
