//! Echo protocol handler
//!
//! Simple request/response protocol that echoes back messages.

use tokio::sync::broadcast;

use crate::cli::daemon::test_protocols::{EchoRequest, EchoResponse, EchoError};
use super::super::{DaemonResponse};

/// Initialize the Echo protocol handler
pub async fn initialize(
    _daemon_key: fastn_id52::SecretKey,
    _response_tx: broadcast::Sender<DaemonResponse>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Initializing Echo protocol handler");
    
    // TODO: Set up actual P2P listener for Echo protocol using fastn_p2p::listen
    // let protocols = [TestProtocol::Echo];
    // let stream = fastn_p2p::listen(daemon_key, &protocols)?;
    // 
    // tokio::spawn(async move {
    //     let mut stream = std::pin::pin!(stream);
    //     while let Some(request_result) = stream.next().await {
    //         let request = request_result?;
    //         fastn_p2p::spawn(async move {
    //             request.handle(echo_handler).await
    //         });
    //     }
    // });
    
    println!("✅ Echo protocol handler ready");
    Ok(())
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