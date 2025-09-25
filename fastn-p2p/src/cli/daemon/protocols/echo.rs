//! Echo protocol handler
//!
//! Simple request/response protocol that echoes back messages.

use crate::cli::daemon::test_protocols::{EchoRequest, EchoResponse, EchoError};
use crate::cli::daemon::protocol_trait::Protocol;

/// Echo protocol implementation
pub struct EchoProtocol;

#[async_trait::async_trait]
impl Protocol for EchoProtocol {
    const NAME: &'static str = "Echo";
    
    async fn init(
        bind_alias: &str,
        config_path: &std::path::PathBuf,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        todo!("Create Echo config directory, write default echo config.json, set up Echo workspace for bind_alias: {}", bind_alias);
    }
    
    async fn load(
        bind_alias: &str,
        config_path: &std::path::PathBuf,
        identity_key: &fastn_id52::SecretKey,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        todo!("Load Echo config from {}, start P2P Echo listener for identity {}, bind_alias: {}", config_path.display(), identity_key.public_key().id52(), bind_alias);
    }
    
    async fn reload(
        bind_alias: &str,
        config_path: &std::path::PathBuf,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        todo!("Reload Echo config from {}, restart Echo services for bind_alias: {}", config_path.display(), bind_alias);
    }
    
    async fn stop(
        bind_alias: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        todo!("Stop Echo protocol services for bind_alias: {}", bind_alias);
    }
    
    async fn check(
        bind_alias: &str,
        config_path: &std::path::PathBuf,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        todo!("Check Echo config at {} for bind_alias: {} - validate config.json, report issues", config_path.display(), bind_alias);
    }
}

/// Handle Echo protocol requests
pub async fn echo_handler(request: EchoRequest) -> Result<EchoResponse, EchoError> {
    println!("📢 Echo request: {}", request.message);
    
    // Simple validation
    if request.message.is_empty() {
        return Err(EchoError::InvalidMessage("Message cannot be empty".to_string()));
    }
    
    if request.message.len() > 1000 {
        return Err(EchoError::InvalidMessage("Message too long (max 1000 chars)".to_string()));
    }
    
    let response = EchoResponse {
        echoed: format!("Echo: {}", request.message),
    };
    
    println!("📤 Echo response: {}", response.echoed);
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_echo_handler() {
        let request = EchoRequest {
            message: "Hello World".to_string(),
        };
        
        let response = echo_handler(request).await.unwrap();
        assert_eq!(response.echoed, "Echo: Hello World");
    }
    
    #[tokio::test]
    async fn test_echo_handler_empty_message() {
        let request = EchoRequest {
            message: "".to_string(),
        };
        
        let result = echo_handler(request).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty"));
    }
}