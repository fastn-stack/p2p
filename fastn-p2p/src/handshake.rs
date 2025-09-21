/// Handshake protocol for fastn-p2p connections
/// 
/// Every connection must complete a handshake before any application protocols can be used.
/// This allows for authentication, protocol negotiation, and client information exchange.

use serde::{Deserialize, Serialize};

/// Handshake error codes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HandshakeError {
    /// Client is not authorized to connect
    Unauthorized,
    /// No common protocols between client and server
    NoCommonProtocols,
    /// Invalid authentication token
    InvalidToken,
    /// Server is at capacity
    ServerFull,
    /// Internal server error
    InternalError,
}

/// Client's initial handshake message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientHello {
    /// Client application name (e.g., "malai", "fastn-cli")
    pub client_name: String,
    
    /// Client version
    pub client_version: String,
    
    /// List of protocols the client wants to use
    pub supported_protocols: Vec<serde_json::Value>,
    
    /// Optional authentication token
    pub auth_token: Option<String>,
}

/// Server's response to ClientHello
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ServerHello {
    Success {
        /// Server name
        server_name: String,
        
        /// Server version
        server_version: String,
        
        /// Protocols accepted by server (subset of client's list)
        accepted_protocols: Vec<serde_json::Value>,
    },
    Failure {
        /// Error code for programmatic handling
        code: HandshakeError,
    },
}

/// The handshake protocol identifier
pub const HANDSHAKE_PROTOCOL: &str = "fastn-p2p-handshake-v1";

impl ClientHello {
    pub fn new(
        client_name: impl Into<String>,
        client_version: impl Into<String>,
    ) -> Self {
        Self {
            client_name: client_name.into(),
            client_version: client_version.into(),
            supported_protocols: Vec::new(),
            auth_token: None,
        }
    }
    
    pub fn with_protocol(mut self, protocol: impl Serialize) -> Self {
        if let Ok(json) = serde_json::to_value(protocol) {
            self.supported_protocols.push(json);
        }
        self
    }
    
    pub fn with_auth(mut self, token: String) -> Self {
        self.auth_token = Some(token);
        self
    }
}

impl ServerHello {
    pub fn success() -> Self {
        Self::Success {
            server_name: "fastn-p2p-server".to_string(),
            server_version: env!("CARGO_PKG_VERSION").to_string(),
            accepted_protocols: Vec::new(),
        }
    }
    
    pub fn failure(code: HandshakeError) -> Self {
        Self::Failure {
            code,
        }
    }
}