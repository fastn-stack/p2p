use std::sync::Arc;
use tokio::sync::OnceCell;

/// Global singleton endpoint instance
static ENDPOINT: OnceCell<Arc<iroh::Endpoint>> = OnceCell::const_new();

/// Get or create the singleton endpoint for this process
/// 
/// This ensures we only have one iroh::Endpoint instance per process,
/// which is the recommended pattern for iroh:
/// - Reuses QUIC connections efficiently
/// - Shares discovery mechanisms
/// - Reduces resource usage (single UDP socket)
/// - Improves connection reliability
pub async fn get_or_create_endpoint(
    secret_key: fastn_id52::SecretKey,
) -> eyre::Result<iroh::Endpoint> {
    // If already initialized, return a cheap clone
    if let Some(endpoint) = ENDPOINT.get() {
        return Ok(endpoint.as_ref().clone());
    }

    // Initialize the endpoint (only happens once)
    let endpoint = ENDPOINT
        .get_or_init(|| async {
            match create_endpoint(secret_key).await {
                Ok(ep) => Arc::new(ep),
                Err(e) => {
                    // Log error and panic since we can't recover from this
                    eprintln!("Failed to create iroh endpoint: {}", e);
                    panic!("Failed to create iroh endpoint: {}", e);
                }
            }
        })
        .await;

    Ok(endpoint.as_ref().clone())
}

/// Create a new iroh endpoint with fastn configuration
async fn create_endpoint(secret_key: fastn_id52::SecretKey) -> eyre::Result<iroh::Endpoint> {
    // Convert fastn_id52::SecretKey to iroh::SecretKey
    let iroh_secret_key = iroh::SecretKey::from_bytes(&secret_key.to_secret_bytes());

    println!("🔧 Creating singleton iroh endpoint...");
    
    match iroh::Endpoint::builder()
        .discovery_n0()
        .discovery_local_network()
        .alpns(vec![crate::APNS_IDENTITY.into()])
        .secret_key(iroh_secret_key)
        .bind()
        .await
    {
        Ok(ep) => {
            println!("✅ Singleton endpoint created: {}", ep.node_id());
            Ok(ep)
        }
        Err(e) => {
            // https://github.com/n0-computer/iroh/issues/2741
            Err(eyre::anyhow!("failed to bind to iroh network: {e:?}"))
        }
    }
}

/// Check if the endpoint has been initialized
pub fn is_initialized() -> bool {
    ENDPOINT.get().is_some()
}

/// Reset the singleton (mainly for testing)
/// 
/// WARNING: This should only be used in tests. In production,
/// the endpoint should live for the entire process lifetime.
#[cfg(test)]
pub async fn reset_singleton() {
    // OnceCell doesn't have a reset method, so we can't actually reset it
    // This is intentional - the singleton should live for the process lifetime
    // For tests, each test process will have its own singleton
}