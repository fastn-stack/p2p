//! Task coordination helpers with strict singleton access control
//!
//! This module encapsulates ALL graceful access and fastn_net::get_stream usage
//! to ensure complete singleton access control.

use crate::client::CallError;

/// Global graceful shutdown coordinator (accessible within crate)
pub(crate) static GRACEFUL: std::sync::LazyLock<fastn_net::Graceful> =
    std::sync::LazyLock::new(fastn_net::Graceful::new);

/// Spawn a task with proper graceful shutdown coordination
///
/// This is the ONLY way to spawn tasks - ensures proper shutdown tracking.
pub fn spawn<F>(task: F) -> tokio::task::JoinHandle<F::Output>
where
    F: std::future::Future + Send + 'static,
    F::Output: Send + 'static,
{
    GRACEFUL.spawn(task)
}

/// Check for graceful shutdown signal
///
/// This is the ONLY way to check for cancellation.
pub async fn cancelled() {
    GRACEFUL.cancelled().await
}

/// Trigger graceful shutdown of all spawned tasks
///
/// This is used by the main macro to initiate shutdown after user main completes
/// or when signal handlers are triggered.
pub async fn shutdown() -> eyre::Result<()> {
    GRACEFUL.shutdown().await
}

/// Internal P2P call implementation with localized graceful access
///
/// This function contains the ONLY internal access to graceful for fastn_net compatibility.
/// All P2P calls go through this function to maintain singleton access control.
pub async fn internal_call<P, INPUT, OUTPUT, ERROR>(
    sender: fastn_id52::SecretKey,
    target: &fastn_id52::PublicKey,
    protocol: P,
    input: INPUT,
) -> Result<Result<OUTPUT, ERROR>, CallError>
where
    P: serde::Serialize
        + for<'de> serde::Deserialize<'de>
        + Clone
        + PartialEq
        + std::fmt::Debug
        + Send
        + Sync
        + 'static,
    INPUT: serde::Serialize,
    OUTPUT: for<'de> serde::Deserialize<'de>,
    ERROR: for<'de> serde::Deserialize<'de>,
{
    // First establish connection and do handshake
    let endpoint = fastn_net::get_endpoint(sender.clone())
        .await
        .map_err(|source| CallError::Endpoint { source })?;
    
    // Connect to target
    let target_node_id = iroh::NodeId::from(
        iroh::PublicKey::from_bytes(&target.to_bytes())
            .map_err(|e| CallError::Stream { source: eyre::Error::from(e) })?
    );
    let conn = endpoint.connect(target_node_id, &fastn_net::APNS_IDENTITY)
        .await
        .map_err(|e| CallError::Stream { source: eyre::Error::from(e) })?;
    
    // Send handshake first
    let handshake_protocol = fastn_net::Protocol::Generic(
        serde_json::Value::String(crate::handshake::HANDSHAKE_PROTOCOL.to_string())
    );
    
    let (mut hs_send, mut hs_recv) = conn.open_bi().await
        .map_err(|e| CallError::Stream { source: eyre::Error::from(e) })?;
    
    // Send handshake protocol identifier
    let protocol_json = serde_json::to_string(&handshake_protocol)
        .map_err(|source| CallError::Serialization { source })?;
    hs_send.write_all(protocol_json.as_bytes()).await
        .map_err(|e| CallError::Send { source: eyre::Error::from(e) })?;
    hs_send.write_all(b"\n").await
        .map_err(|e| CallError::Send { source: eyre::Error::from(e) })?;
    
    // Wait for ACK
    let ack = fastn_net::next_string(&mut hs_recv).await
        .map_err(|source| CallError::Receive { source })?;
    if ack != fastn_net::ACK {
        return Err(CallError::Receive { 
            source: eyre::anyhow!("Expected ACK, got: {}", ack) 
        });
    }
    
    // Send ClientHello
    let client_hello = crate::handshake::ClientHello::new(
        "fastn-p2p-client",
        env!("CARGO_PKG_VERSION")
    ).with_protocol(&protocol);
    
    let hello_json = serde_json::to_string(&client_hello)
        .map_err(|source| CallError::Serialization { source })?;
    hs_send.write_all(hello_json.as_bytes()).await
        .map_err(|e| CallError::Send { source: eyre::Error::from(e) })?;
    hs_send.write_all(b"\n").await
        .map_err(|e| CallError::Send { source: eyre::Error::from(e) })?;
    
    // Read ServerHello
    let server_hello: crate::handshake::ServerHello = fastn_net::next_json(&mut hs_recv).await
        .map_err(|source| CallError::Receive { source })?;
    
    // Check if handshake succeeded
    let accepted_protocols = match server_hello {
        crate::handshake::ServerHello::Success { 
            accepted_protocols, ..
        } => accepted_protocols,
        crate::handshake::ServerHello::Failure { code } => {
            return Err(CallError::Receive { 
                source: eyre::anyhow!("Server rejected handshake: {:?}", code)
            });
        }
    };
    
    // Check if our protocol is accepted
    let protocol_json = serde_json::to_value(&protocol)
        .map_err(|e| CallError::Serialization { source: e })?;
    if !accepted_protocols.contains(&protocol_json) {
        return Err(CallError::Receive { 
            source: eyre::anyhow!("Server doesn't support requested protocol")
        });
    }
    
    hs_send.finish()
        .map_err(|e| CallError::Send { source: eyre::Error::from(e) })?;
    
    // Now open the actual application protocol stream
    let app_protocol = fastn_net::Protocol::Generic(serde_json::Value::String("fastn-p2p".to_string()));
    
    let (mut send_stream, mut recv_stream) = conn.open_bi().await
        .map_err(|e| CallError::Stream { source: eyre::Error::from(e) })?;
    
    // Send app protocol identifier  
    let app_protocol_json = serde_json::to_string(&app_protocol)
        .map_err(|source| CallError::Serialization { source })?;
    send_stream.write_all(app_protocol_json.as_bytes()).await
        .map_err(|e| CallError::Send { source: eyre::Error::from(e) })?;
    send_stream.write_all(b"\n").await
        .map_err(|e| CallError::Send { source: eyre::Error::from(e) })?;
    
    // Wait for ACK
    let ack = fastn_net::next_string(&mut recv_stream).await
        .map_err(|source| CallError::Receive { source })?;
    if ack != fastn_net::ACK {
        return Err(CallError::Receive { 
            source: eyre::anyhow!("Expected ACK for app protocol, got: {}", ack) 
        });
    }

    // Convert user protocol to JSON for embedding in request
    let protocol_json =
        serde_json::to_value(&protocol).map_err(|e| CallError::Serialization { source: e })?;

    // Create wrapper request with protocol and data
    let wrapper_request = serde_json::json!({
        "protocol": protocol_json,
        "data": input
    });
    let request_json = serde_json::to_string(&wrapper_request)
        .map_err(|source| CallError::Serialization { source })?;

    // Send JSON followed by newline
    send_stream
        .write_all(request_json.as_bytes())
        .await
        .map_err(|e| CallError::Send {
            source: eyre::Error::from(e),
        })?;
    send_stream
        .write_all(b"\n")
        .await
        .map_err(|e| CallError::Send {
            source: eyre::Error::from(e),
        })?;

    // Receive and deserialize response
    // We use next_string here because we need to try deserializing as two different types
    let response_json = fastn_net::next_string(&mut recv_stream)
        .await
        .map_err(|source| CallError::Receive { source })?;

    // Try to deserialize as success response first
    if let Ok(success_response) = serde_json::from_str::<OUTPUT>(&response_json) {
        return Ok(Ok(success_response));
    }

    // If that fails, try to deserialize as ERROR type
    if let Ok(error_response) = serde_json::from_str::<ERROR>(&response_json) {
        return Ok(Err(error_response));
    }

    // If both fail, it's a deserialization error
    Err(CallError::Deserialization {
        source: serde_json::Error::io(std::io::Error::other(format!(
            "Response doesn't match expected OUTPUT or ERROR types: {response_json}"
        ))),
    })
}
