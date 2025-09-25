//! Error types for fastn-p2p-client

/// Client operation errors
#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("Daemon connection failed: {0}")]
    DaemonConnection(String),

    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("Serialization error: {source}")]
    Serialization { 
        #[from]
        source: serde_json::Error 
    },

    #[error("IO error: {source}")]
    Io { 
        #[from]
        source: std::io::Error 
    },

    #[error("Configuration error: {0}")]
    Configuration(String),
}

/// Connection errors for streaming operations
#[derive(Debug, thiserror::Error)]
pub enum ConnectionError {
    #[error("Failed to connect to daemon: {0}")]
    DaemonConnection(String),
    
    #[error("Failed to establish stream: {0}")]
    StreamSetup(String),
    
    #[error("Serialization error: {source}")]
    Serialization { 
        #[from]
        source: serde_json::Error 
    },
    
    #[error("IO error: {source}")]
    Io { 
        #[from]
        source: std::io::Error 
    },
}