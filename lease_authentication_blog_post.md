# Secure P2P Authentication with Lease Tokens in fastn-p2p

*Introducing a revolutionary approach to identity management and access control in peer-to-peer applications*

When building peer-to-peer applications, one of the biggest challenges is managing identity and access control securely. How do you allow multiple devices or services to act on behalf of an identity without exposing secret keys? How do you implement time-bound permissions that can be revoked instantly?

Today, I'm excited to introduce **Lease-Based Authentication** in fastn-p2p – a cryptographically secure solution that elegantly solves these problems while keeping the developer experience simple and intuitive.

## The Problem: Identity vs Access

Traditional P2P systems face a fundamental dilemma:

```rust
// ❌ The old way: Sharing secret keys is dangerous
let shared_secret = load_secret_key(); // Same key everywhere!
let response = fastn_p2p::call(shared_secret, server, MyProtocol::Deploy, request).await?;
```

Sharing secret keys across devices creates massive security risks:
- **No revocation**: Can't revoke access without changing the identity
- **All-or-nothing permissions**: Every device has full access
- **No audit trail**: Can't track which device did what
- **Permanent exposure**: Compromised keys require identity migration

## The Solution: Cryptographic Leases

Lease-based authentication separates **identity ownership** from **access delegation**:

```rust
// ✅ The new way: Secure delegation with time bounds
let identity_owner = SecretKey::load_from_keyring("production-identity")?;
let ci_device = SecretKey::generate(); // CI system's own key

// Identity owner creates a time-bound lease for the CI system
let lease = identity_owner.create_lease(
    ci_device.public_key(),           // Device getting access
    Duration::from_secs(30 * 60),     // 30-minute window
    Some("deploy:production".into())   // Scoped permissions
);

// CI system uses its own key + the lease to act as the identity
let response = fastn_p2p::call(
    ci_device,
    production_server,
    DeployProtocol::Deploy,
    deployment_request,
    Some(lease) // ← The magic happens here
).await?;
```

## Core Concepts

### 1. Signed Data Pattern

At the heart of the system is a generic `SignedData<T>` type that provides cryptographic proof of authenticity:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedData<T> {
    pub content: T,
    pub signature: Signature,
    pub signed_by: PublicKey,
}

impl<T> SignedData<T> {
    /// Create cryptographically signed data
    pub fn sign(content: T, signer: &SecretKey) -> Self;
    
    /// Verify signature and get content
    pub fn verified_content(&self) -> Result<&T, SignatureError>;
    
    /// Revalidate signature (for paranoid applications)
    pub fn revalidate(&self) -> Result<(), SignatureError>;
}
```

### 2. Lease Token Structure

A lease token is simply signed lease data:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaseData {
    /// The identity being delegated
    pub identity_key: PublicKey,
    /// Device authorized to use this identity
    pub device_key: PublicKey,
    /// When this lease expires (Unix timestamp)
    pub expires_at: u64,
    /// Optional scoped permissions
    pub scope: Option<String>,
    /// When lease was issued
    pub issued_at: u64,
}

// A lease token is just signed lease data
pub type LeaseToken = SignedData<LeaseData>;
```

### 3. Connection-First Architecture

With explicit authentication, we use a connection-first pattern for efficiency:

```rust
pub struct AuthenticatedConnection {
    // Internal connection details
}

impl AuthenticatedConnection {
    /// Establish authenticated connection once
    pub async fn connect(
        our_key: SecretKey,
        target: PublicKey,
        lease_token: Option<LeaseToken>,
    ) -> Result<Self, ConnectionError>;
    
    /// Make multiple calls on the same connection
    pub async fn call<P, Req, Resp, Err>(&mut self, protocol: P, request: Req) 
        -> Result<Result<Resp, Err>, CallError>;
    
    /// Open streaming session
    pub async fn stream<P, D>(&mut self, protocol: P, data: D) 
        -> Result<Session, ConnectionError>;
}
```

## Complete API Reference

Let's look at the full public API surface after lease integration:

### Core Types

```rust
// Re-exported from fastn-id52
pub use fastn_id52::{SecretKey, PublicKey, Signature};

// New lease types
pub struct SignedData<T> { /* ... */ }
pub struct LeaseData { /* ... */ }
pub type LeaseToken = SignedData<LeaseData>;

// Connection types
pub struct AuthenticatedConnection { /* ... */ }
pub struct Session { /* ... */ }

// Error types
pub enum ConnectionError { /* ... */ }
pub enum CallError { /* ... */ }
pub enum SignatureError { /* ... */ }
```

### Identity Management

```rust
impl SecretKey {
    /// Generate a new cryptographically secure identity
    pub fn generate() -> Self;
    
    /// Load from system keyring
    pub fn from_keyring(id52: &str) -> Result<Self, KeyringError>;
    
    /// Create a lease for another device
    pub fn create_lease(
        &self,
        device_public_key: PublicKey,
        duration: Duration,
        scope: Option<String>,
    ) -> LeaseToken;
    
    /// Get the public key for this identity
    pub fn public_key(&self) -> PublicKey;
    
    /// Get ID52 string representation
    pub fn id52(&self) -> String;
}
```

### Client API - Simple Operations

```rust
/// Simple request/response (convenience function)
pub async fn call<PROTOCOL, REQUEST, RESPONSE, ERROR>(
    our_key: SecretKey,
    target: PublicKey,
    protocol: PROTOCOL,
    request: REQUEST,
    lease_token: Option<LeaseToken>,
) -> Result<Result<RESPONSE, ERROR>, CallError>;

/// Simple streaming connection (convenience function)
pub async fn connect<PROTOCOL, DATA>(
    our_key: SecretKey,
    target: PublicKey,
    protocol: PROTOCOL,
    data: DATA,
    lease_token: Option<LeaseToken>,
) -> Result<Session, ConnectionError>;
```

### Client API - Connection-First (Recommended)

```rust
impl AuthenticatedConnection {
    /// Establish authenticated connection
    pub async fn connect(
        our_key: SecretKey,
        target: PublicKey,
        lease_token: Option<LeaseToken>,
    ) -> Result<Self, ConnectionError>;
    
    /// Make authenticated call
    pub async fn call<PROTOCOL, REQUEST, RESPONSE, ERROR>(
        &mut self,
        protocol: PROTOCOL,
        request: REQUEST,
    ) -> Result<Result<RESPONSE, ERROR>, CallError>;
    
    /// Open streaming session
    pub async fn stream<PROTOCOL, DATA>(
        &mut self,
        protocol: PROTOCOL,
        data: DATA,
    ) -> Result<Session, ConnectionError>;
    
    /// Get peer information
    pub fn peer(&self) -> &PublicKey;
    
    /// Check if lease was validated
    pub fn lease_validated(&self) -> bool;
}
```

### Server API

```rust
/// Listen for incoming connections
pub fn listen<P>(
    secret_key: SecretKey,
    expected_protocols: &[P],
) -> Result<impl Stream<Item = Result<Request<P>, eyre::Error>>, ListenerAlreadyActiveError>;

/// Enhanced server builder with lease validation
pub struct ServerBuilder;

impl ServerBuilder {
    pub fn new(secret_key: SecretKey) -> Self;
    
    /// Add lease validator
    pub fn with_lease_validator<V>(self, validator: V) -> Self 
    where V: LeaseValidator + 'static;
    
    /// Add connection-level authentication
    pub fn with_connection_auth<F>(self, auth_fn: F) -> Self
    where F: Fn(&PublicKey) -> bool + Send + Sync + 'static;
    
    /// Add protocol-level authorization
    pub fn with_stream_auth<F>(self, auth_fn: F) -> Self
    where F: Fn(&PublicKey, &serde_json::Value, &str) -> bool + Send + Sync + 'static;
    
    /// Register protocol handler
    pub fn handle_requests<P, Req, Resp, Err, F, Fut>(
        self, 
        protocol: P, 
        handler: F
    ) -> Self
    where F: Fn(Req) -> Fut + Send + Sync + 'static,
          Fut: Future<Output = Result<Resp, Err>> + Send;
    
    /// Start the server
    pub async fn serve(self) -> Result<(), eyre::Error>;
}

/// Lease validation trait
pub trait LeaseValidator: Send + Sync {
    fn validate_lease(&self, token: &LeaseToken) -> LeaseValidationResult;
    fn is_revoked(&self, token: &LeaseToken) -> bool;
}

#[derive(Debug)]
pub enum LeaseValidationResult {
    Valid,
    Expired,
    InvalidSignature,
    Revoked,
    ScopeNotAllowed,
}
```

## Common Usage Patterns

### Pattern 1: CI/CD Deployment

```rust
use fastn_p2p::*;
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
enum DeployProtocol {
    Deploy,
    Status,
    Rollback,
}

#[derive(Serialize, Deserialize)]
struct DeployRequest {
    app_name: String,
    version: String,
    environment: String,
}

// Production identity owner creates lease for CI system
async fn create_ci_lease() -> Result<LeaseToken, Box<dyn std::error::Error>> {
    let prod_identity = SecretKey::from_keyring("production-deployer")?;
    let ci_public_key = PublicKey::from_str("ci-system-id52")?;
    
    let lease = prod_identity.create_lease(
        ci_public_key,
        Duration::from_secs(30 * 60), // 30 minutes
        Some("deploy:production".into())
    );
    
    // Securely distribute lease to CI system
    Ok(lease)
}

// CI system uses the lease to deploy
async fn deploy_with_lease(
    lease_token: LeaseToken,
    target_server: PublicKey,
) -> Result<(), Box<dyn std::error::Error>> {
    let ci_key = SecretKey::from_keyring("ci-system")?;
    
    // Establish authenticated connection
    let mut conn = AuthenticatedConnection::connect(
        ci_key,
        target_server,
        Some(lease_token),
    ).await?;
    
    // Make deployment call
    let deploy_request = DeployRequest {
        app_name: "my-app".into(),
        version: "v1.2.3".into(),
        environment: "production".into(),
    };
    
    let result: Result<DeployResponse, DeployError> = conn.call(
        DeployProtocol::Deploy,
        deploy_request,
    ).await?;
    
    match result {
        Ok(response) => println!("Deployment successful: {:?}", response),
        Err(err) => println!("Deployment failed: {:?}", err),
    }
    
    Ok(())
}
```

### Pattern 2: Multi-Device Identity

```rust
// User's main device creates leases for other devices
async fn setup_multi_device() -> Result<(), Box<dyn std::error::Error>> {
    let main_device = SecretKey::from_keyring("user-main-identity")?;
    
    // Create lease for mobile device (shorter duration)
    let mobile_key = PublicKey::from_str("mobile-device-id52")?;
    let mobile_lease = main_device.create_lease(
        mobile_key,
        Duration::from_secs(24 * 60 * 60), // 24 hours
        Some("sync:read-only".into())
    );
    
    // Create lease for laptop (longer duration, more permissions)
    let laptop_key = PublicKey::from_str("laptop-device-id52")?;
    let laptop_lease = main_device.create_lease(
        laptop_key,
        Duration::from_secs(7 * 24 * 60 * 60), // 7 days
        Some("sync:read-write".into())
    );
    
    // Distribute leases securely to respective devices
    distribute_lease_to_mobile(mobile_lease).await?;
    distribute_lease_to_laptop(laptop_lease).await?;
    
    Ok(())
}

// Mobile device syncs using its lease
async fn mobile_sync(lease_token: LeaseToken) -> Result<(), Box<dyn std::error::Error>> {
    let mobile_key = SecretKey::from_device_keychain()?;
    let sync_server = PublicKey::from_str("sync-server-id52")?;
    
    let mut conn = AuthenticatedConnection::connect(
        mobile_key,
        sync_server,
        Some(lease_token),
    ).await?;
    
    // Sync data with read-only permissions
    let sync_data: SyncData = conn.call(
        SyncProtocol::PullUpdates,
        PullRequest { since: last_sync_time() },
    ).await??;
    
    apply_sync_data(sync_data).await?;
    Ok(())
}
```

### Pattern 3: Server with Lease Validation

```rust
use std::collections::HashSet;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
struct ProductionLeaseValidator {
    revoked_leases: Arc<RwLock<HashSet<String>>>,
    allowed_scopes: Vec<String>,
    max_lease_duration: Duration,
}

impl LeaseValidator for ProductionLeaseValidator {
    fn validate_lease(&self, token: &LeaseToken) -> LeaseValidationResult {
        let lease_data = match token.verified_content() {
            Ok(data) => data,
            Err(_) => return LeaseValidationResult::InvalidSignature,
        };
        
        // Check expiration
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        if lease_data.expires_at < now {
            return LeaseValidationResult::Expired;
        }
        
        // Check lease duration policy
        let lease_duration = lease_data.expires_at - lease_data.issued_at;
        if lease_duration > self.max_lease_duration.as_secs() {
            return LeaseValidationResult::ScopeNotAllowed;
        }
        
        // Check scope permissions
        if let Some(scope) = &lease_data.scope {
            if !self.allowed_scopes.contains(scope) {
                return LeaseValidationResult::ScopeNotAllowed;
            }
        }
        
        LeaseValidationResult::Valid
    }
    
    fn is_revoked(&self, token: &LeaseToken) -> bool {
        if let Ok(lease_data) = token.verified_content() {
            let lease_id = format!("{}:{}", 
                lease_data.device_key.id52(), 
                lease_data.issued_at
            );
            self.revoked_leases.read().unwrap().contains(&lease_id)
        } else {
            true // Invalid signatures are considered revoked
        }
    }
}

async fn run_production_server() -> Result<(), Box<dyn std::error::Error>> {
    let server_key = SecretKey::from_keyring("production-server")?;
    
    let validator = ProductionLeaseValidator {
        revoked_leases: Arc::new(RwLock::new(HashSet::new())),
        allowed_scopes: vec![
            "deploy:production".into(),
            "deploy:staging".into(),
            "sync:read-only".into(),
            "sync:read-write".into(),
        ],
        max_lease_duration: Duration::from_secs(24 * 60 * 60), // Max 24 hours
    };
    
    ServerBuilder::new(server_key)
        .with_lease_validator(validator)
        .with_connection_auth(|peer| {
            // Allow connections from known networks
            is_from_trusted_network(peer)
        })
        .handle_requests(DeployProtocol::Deploy, handle_deploy)
        .handle_requests(DeployProtocol::Status, handle_status)
        .handle_requests(SyncProtocol::PullUpdates, handle_sync)
        .serve()
        .await?;
    
    Ok(())
}
```

### Pattern 4: Lease Revocation

```rust
// Emergency revocation system
async fn revoke_compromised_device() -> Result<(), Box<dyn std::error::Error>> {
    let device_to_revoke = PublicKey::from_str("compromised-device-id52")?;
    
    // Add to revocation list (shared across all servers)
    let revocation_service = RevocationService::connect().await?;
    revocation_service.revoke_device(device_to_revoke).await?;
    
    // All servers will reject future connections from this device
    println!("Device {} has been revoked", device_to_revoke.id52());
    
    Ok(())
}
```

## Best Practices

### 1. Lease Duration Guidelines

```rust
// ✅ Good: Use appropriate durations for use case
let ci_lease = identity.create_lease(
    ci_device,
    Duration::from_secs(30 * 60),  // 30 min for deployments
    Some("deploy:production".into())
);

let mobile_lease = identity.create_lease(
    mobile_device,
    Duration::from_secs(24 * 60 * 60),  // 24 hours for user devices
    Some("sync:read-only".into())
);

// ❌ Bad: Overly long leases reduce security
let bad_lease = identity.create_lease(
    device,
    Duration::from_secs(365 * 24 * 60 * 60),  // 1 year is too long!
    None
);
```

### 2. Scope Design

```rust
// ✅ Good: Granular, hierarchical scopes
"deploy:production"
"deploy:staging"
"sync:read-only"
"sync:read-write"
"admin:user-management"
"admin:system-config"

// ❌ Bad: Vague or overly broad scopes
"admin"  // Too broad
"deploy" // Missing environment
"access" // Meaningless
```

### 3. Connection Reuse

```rust
// ✅ Good: Reuse authenticated connections
let mut conn = AuthenticatedConnection::connect(key, target, lease).await?;

for request in batch_requests {
    let response = conn.call(protocol, request).await?;
    process_response(response).await?;
}

// ❌ Bad: New connection per request
for request in batch_requests {
    let response = fastn_p2p::call(key, target, protocol, request, lease.clone()).await?;
    process_response(response).await?;
}
```

### 4. Error Handling

```rust
// ✅ Good: Handle lease-specific errors
match conn.call(protocol, request).await {
    Ok(Ok(response)) => process_success(response),
    Ok(Err(app_error)) => handle_application_error(app_error),
    Err(CallError::Unauthorized) => {
        // Lease might be expired or revoked
        refresh_lease_and_retry().await?
    },
    Err(other) => handle_network_error(other),
}
```

### 5. Security Considerations

```rust
// ✅ Good: Validate leases in production
impl LeaseValidator for ProductionValidator {
    fn validate_lease(&self, token: &LeaseToken) -> LeaseValidationResult {
        // Always revalidate signature
        if token.revalidate().is_err() {
            return LeaseValidationResult::InvalidSignature;
        }
        
        // Check business rules
        let data = token.verified_content().unwrap();
        if self.is_scope_allowed(&data.scope) && 
           self.is_duration_acceptable(&data) &&
           !self.is_revoked(token) {
            LeaseValidationResult::Valid
        } else {
            LeaseValidationResult::ScopeNotAllowed
        }
    }
}

// ✅ Good: Secure lease distribution
async fn distribute_lease_securely(lease: LeaseToken, target: &str) {
    // Use encrypted channels for lease distribution
    let encrypted_lease = encrypt_for_recipient(lease, target)?;
    secure_channel_send(encrypted_lease, target).await?;
}
```

## Migration Guide

Existing fastn-p2p applications can migrate gradually:

### Phase 1: Add Optional Lease Support

```rust
// Old code still works
let response = fastn_p2p::call(key, target, protocol, request, None).await?;

// New code with leases
let response = fastn_p2p::call(key, target, protocol, request, Some(lease)).await?;
```

### Phase 2: Implement Lease Validation

```rust
// Server adds lease validator gradually
ServerBuilder::new(key)
    .with_lease_validator(BasicLeaseValidator::new())
    .handle_requests(protocol, handler)
    .serve().await?;
```

### Phase 3: Enforce Lease Requirements

```rust
// Eventually require leases for sensitive operations
impl StrictLeaseValidator {
    fn validate_lease(&self, token: &LeaseToken) -> LeaseValidationResult {
        // Require leases for all connections
        // No more None lease tokens accepted
    }
}
```

## Conclusion

Lease-based authentication in fastn-p2p provides enterprise-grade security with a developer-friendly API. Key benefits:

- **🔒 Zero-trust security**: Never share secret keys
- **⏰ Time-bound access**: Automatic expiration
- **🎯 Scoped permissions**: Granular access control  
- **🚫 Instant revocation**: Immediate access removal
- **📊 Full audit trail**: Track all access grants
- **🔄 Backward compatible**: Gradual migration path

The `SignedData<T>` pattern and connection-first architecture make the system both secure and performant, while keeping the learning curve minimal for developers.

Ready to get started? Check out the [fastn-p2p documentation](https://docs.rs/fastn-p2p) and join our [community discussions](https://github.com/fastn-stack/p2p/discussions) to share your use cases and get help implementing lease-based authentication in your applications.

---

*Have questions or feedback? Open an issue on [GitHub](https://github.com/fastn-stack/p2p) or join our [Discord community](https://discord.gg/fastn).*