# Identity Leasing in fastn-p2p

Modern P2P applications need secure identity delegation. How do you let a CI system deploy on your behalf without sharing your secret key? How do you give your mobile app access to your data with automatic expiration?

fastn-p2p solves this with **identity leasing** - a cryptographic system where identity owners (grantors) can authorize other entities (grantees) to act on their behalf through time-bound, revocable leases.

## Core Concepts

**Identity**: A cryptographic identity loaded from `myapp.private-key` + `myapp.sqlite`

**Lease Permission**: Standing authorization ("mobile app CAN request 7-day user:read leases")

**Live Lease**: Active instance ("mobile app HAS lease #123 until tomorrow")

**Grantor**: Identity owner who grants lease permissions

**Grantee**: Entity that requests and uses leases

**Verifier**: Server that validates leases (happens automatically)

## Basic Usage

### Grantor: Set up permissions

```rust
let grantor = Identity::load("production")?;

// Mobile app can auto-issue 7-day leases
grantor.allow_leases("mobile-app-id52", "7d", &["user:read"], true).await?;

// CI system needs approval for each lease
grantor.allow_leases("ci-system-id52", "1h", &["deploy:staging"], false).await?;
```

### Grantee: Request and use leases

```rust
let grantee = Identity::load("mobile-app")?;

// Request a lease (auto-approved if permission exists)
let lease_id = grantee.request_lease("production-id52", "24h", "user:read").await?;

// Use the lease
let conn = Connection::connect(grantee, "api-server-id52", lease_id).await?;
let data = conn.call(UserProtocol::GetData, request).await?;
```

### Server: Automatic verification

```rust
let server = Identity::load("api-server")?;

Server::listen(server)
    .handle(UserProtocol::GetData, handle_get_data)
    .serve().await?;

// Lease verification happens automatically
async fn handle_get_data(request: GetDataRequest) -> Result<UserData, DataError> {
    // Request is already authenticated via lease
    fetch_user_data(request.user_id).await
}
```

## Real-World Scenarios

### Scenario 1: Multi-Device User

**Problem**: User wants their mobile app and laptop to access their cloud data without sharing the main identity key.

**Solution**:
```rust
// Main device sets up permissions
let main = Identity::load("user-main")?;

// Mobile gets limited access
main.allow_leases("mobile-id52", "1d", &["sync:read"], true).await?;

// Laptop gets broader access  
main.allow_leases("laptop-id52", "30d", &["sync:*", "settings:*"], true).await?;

// Devices auto-issue leases as needed
let mobile = Identity::load("mobile")?;
let lease = mobile.request_lease("user-main-id52", "8h", "sync:read").await?;
```

### Scenario 2: CI/CD Pipeline

**Problem**: Deployment pipeline needs to act as production identity for deployments, but with strict time limits and approval workflow.

**Solution**:
```rust
// DevOps admin sets up CI permissions
let admin = Identity::load("devops-admin")?;

// CI can request short deployment leases (requires approval)
admin.allow_leases("ci-system-id52", "30m", &["deploy:*"], false).await?;

// CI requests lease for specific deployment
let ci = Identity::load("ci-system")?;
let lease_request = ci.request_lease("devops-admin-id52", "15m", "deploy:production").await?;

// Admin approves via web interface or CLI
admin.approve_lease_request(lease_request).await?;

// CI deploys using approved lease
let conn = Connection::connect(ci, "prod-server-id52", lease_request).await?;
conn.call(DeployProtocol::Deploy, deploy_config).await?;
```

### Scenario 3: Partner API Access

**Problem**: Business partner needs API access to your service, but you want granular control and audit trails.

**Solution**:
```rust
// Your service sets up partner permissions
let service = Identity::load("my-service")?;

// Partner can auto-issue 90-day API leases
service.allow_leases("partner-id52", "90d", &["api:read", "webhooks:*"], true).await?;

// Partner requests long-lived lease
let partner = Identity::load("partner-system")?;
let api_lease = partner.request_lease("my-service-id52", "60d", "api:read").await?;

// Partner uses lease for API calls
let conn = Connection::connect(partner, "api-gateway-id52", api_lease).await?;
let customers = conn.call(PartnerAPI::ListCustomers, list_request).await?;

// You can monitor and revoke if needed
let usage = service.query().lease_usage(api_lease).await?;
if usage.suspicious() {
    service.revoke_lease(api_lease).await?;
}
```

### Scenario 4: Temporary Team Access

**Problem**: External contractor needs temporary access to internal systems during project work.

**Solution**:
```rust
// Team lead sets up contractor permissions
let team_lead = Identity::load("team-lead")?;

// Contractor can request daily leases for project duration
team_lead.allow_leases(
    "contractor-id52", 
    "1d",                    // Max 1-day leases
    &["project:read", "docs:*"], 
    true                     // Auto-approve for convenience
).await?;

// Contractor requests access each day
let contractor = Identity::load("contractor")?;
let daily_lease = contractor.request_lease("team-lead-id52", "8h", "project:read").await?;

// Access internal systems
let conn = Connection::connect(contractor, "internal-api-id52", daily_lease).await?;
let project_data = conn.call(ProjectAPI::GetSpecs, specs_request).await?;
```

## Benefits

**Security**: Never share secret keys. All access is time-bound and revocable.

**Flexibility**: Support both auto-approved convenience and manual approval workflows.

**Audit**: Complete history of who accessed what, when, and how.

**Simplicity**: 5 concepts, 8 operations. Follows existing fastn file conventions.

**Performance**: Verifiers cache grantor responses. Grantees reuse connections.

## Getting Started

```rust
use fastn_p2p::*;

// Load your identity
let identity = Identity::load("myapp")?;

// Set up a permission
identity.allow_leases("trusted-id52", "1d", &["api:read"], true).await?;

// Request a lease
let lease = identity.request_lease("owner-id52", "2h", "api:read").await?;

// Use it
let conn = Connection::connect(identity, "server-id52", lease).await?;
let result = conn.call(MyProtocol::GetData, request).await?;
```

Identity leasing transforms P2P applications from "all-or-nothing" key sharing to sophisticated, enterprise-grade access control with zero configuration overhead.

---

*Built on Ed25519 cryptography with SQLite storage. Convention-based configuration following fastn patterns.*