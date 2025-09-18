# malai-next: Production-Grade HTTP over P2P

Fresh implementation of malai's HTTP proxy functionality (expose_http/http_bridge) using the new fastn-p2p API. 

**Scope**: Only HTTP over P2P features - not the full malai feature set.

## Overview

malai-next implements production-grade HTTP over P2P:
- **expose_http**: P2P server that exposes local HTTP services  
- **http_bridge**: P2P client that creates local HTTP→P2P bridges

**Goal**: Feature parity with malai's expose_http/http_bridge commands using superior fastn-p2p API.

## Essential Features Analysis

Based on analysis of the current malai implementation, the following production features are essential:

### ✅ Currently Implemented
1. **Basic HTTP Proxying** - Bidirectional HTTP forwarding over P2P
2. **Custom Error Types** - HttpError, BridgeError with specific variants
3. **Efficient Streaming** - session.copy_both() for zero-copy bidirectional I/O
4. **Clean API** - Uses new fastn_p2p::listen().handle_streams() pattern

### 🚧 Missing HTTP-Specific Features (TODO)

**Scope**: Only features related to HTTP over P2P (expose_http/http_bridge)

#### Critical for HTTP Production Use:
1. **HTTP Connection Pooling** - bb8::Pool for connection reuse (performance critical)
2. **Keyring Integration** - System keyring for secure key storage (`use_keyring = true`) 
3. **Persistent Key Management** - File-based fallback when keyring unavailable
4. **Basic Configuration** - TOML config for host/port/permissions
5. **Access Control** - Permission system for HTTP service access

### 🔧 HTTP Connection Pooling (Priority #1)

Current malai uses sophisticated bb8-based connection pooling:

```rust
pub type HttpConnectionPool = bb8::Pool<HttpConnectionManager>;
pub type HttpConnectionPools = Arc<Mutex<HashMap<String, HttpConnectionPool>>>;

// Per-address connection pools for efficiency
let pool = get_pool(addr, client_pools).await?;
let mut client = pool.get().await?;
```

**This MUST be implemented in malai-next** for production performance.

### 🔍 Detailed Feature Analysis

#### Current malai Production Features:

1. **HTTP Connection Pooling** (`kulfi-utils/src/http_connection_manager.rs`)
   - bb8::Pool<HttpConnectionManager> for connection reuse
   - Per-address connection pools (HashMap<String, Pool>)
   - Automatic connection lifecycle management
   - Significant performance improvement for multiple requests

2. **Graceful Shutdown** (`expose_http.rs`)
   - kulfi_utils::Graceful integration
   - tokio::select! with graceful.cancelled()
   - Proper cleanup and signal handling
   - Connection draining before shutdown

3. **Configuration System** (`core/config.rs`)
   - TOML-based configuration with validation
   - Machine permissions and access control
   - Service definitions with ports and permissions
   - Group-based permission management

4. **Key Management** (multiple files)
   - Persistent key storage in files
   - Key validation and error handling  
   - Identity resolution for machines

5. **Service Management** (`core/config.rs`)
   - Multiple HTTP services per machine
   - Port configuration and service discovery
   - Permission-based access control per service

6. **Error Handling & Observability**
   - Comprehensive tracing throughout
   - Error context and proper error propagation
   - Performance timing and metrics

#### Critical Missing Features in malai-next:

**Core Missing (must implement for MVP):**
1. **❌ No HTTP connection pooling** - Creates new TCP connection per request
2. **❌ No keyring integration** - Should use system keyring like malai
3. **❌ No TCP proxy support** - Only HTTP, missing TCP forwarding
4. **❌ No daemon architecture** - No background daemon + Unix socket IPC
5. **❌ No configuration system** - Hardcoded host/port, no TOML config
6. **❌ No persistent keys** - Generates new key each run (not production-ready)
7. **❌ No access control** - Anyone can connect (security hole)
8. **❌ No service management** - Single service only

**Advanced Missing (for feature parity):**
9. **❌ No agent architecture** - No client-side connection pooling
10. **❌ No cluster management** - No multi-machine coordination
11. **❌ No identity management** - No complex identity resolution
12. **❌ No status/monitoring** - No operational visibility commands
13. **❌ No browse integration** - No web interface connectivity
14. **❌ No folder operations** - No file management over P2P
15. **❌ No CLI integration** - No rich CLI with daemon communication

### 📋 Implementation Roadmap

#### Phase 1: Critical Production Features (Required for MVP)
- [ ] **HTTP connection pooling** - bb8::Pool<HttpConnectionManager> (performance critical)
- [ ] **Persistent key management** - File-based key storage like malai
- [x] **Graceful shutdown** - fastn-context handles this automatically via #[fastn_p2p::main]
- [ ] **Basic configuration** - TOML config for services and permissions
- [ ] **Access control** - Permission system for who can access what

#### Phase 2: Production Hardening  
- [ ] **Multiple services** - Single server exposing multiple HTTP services
- [ ] **Service discovery** - Automatic discovery of available services
- [ ] **Health monitoring** - Monitor upstream service availability
- [ ] **Enhanced observability** - Metrics, structured logging
- [ ] **Error resilience** - Circuit breakers, retries, timeouts

#### Phase 3: Ecosystem Integration
- [ ] **DNS integration** - Domain-based addressing like current malai
- [ ] **Cluster management** - Multi-machine coordination
- [ ] **Service mesh features** - Load balancing, routing rules
- [ ] **Security hardening** - Rate limiting, DDoS protection

### 🚨 Current Gap Analysis

**malai-next is currently a DEMO, not production-ready.** It lacks:

1. **Performance** - No connection pooling (creates new TCP per request)
2. **Security** - No access control (anyone can connect)  
3. **Reliability** - Basic error recovery (graceful shutdown handled by fastn-context)
4. **Configuration** - Hardcoded values, no service management
5. **Observability** - Basic logging only, no metrics
6. **Key Management** - Generates new keys (not persistent)

### 🎯 fastn-context Integration

**Graceful shutdown is handled automatically** by fastn-context:

- **#[fastn_p2p::main]** (re-exports fastn_context::main) provides:
  - Automatic signal handling (SIGINT, SIGTERM)
  - Hierarchical context tree for operation tracking
  - Built-in graceful shutdown coordination
  - Context propagation for observability

**This means malai-next gets graceful shutdown for free** - one less thing to implement!

**HTTP Feature Gap Analysis:**

Current malai HTTP functionality is production-grade with:
- Sophisticated bb8 connection pooling
- Keyring integration with file fallback
- Proper HTTP/1.1 streaming with hyper
- Connection lifecycle management

**malai-next HTTP functionality is demo-grade:**
- ✅ Basic HTTP forwarding works
- ✅ Uses superior fastn-p2p API  
- ✅ Efficient session.copy_both() streaming
- ❌ No connection pooling (performance issue)
- ❌ No keyring (generates keys each run)
- ❌ No configuration (hardcoded values)

**For HTTP production parity, malai-next needs:**
1. **bb8 HTTP connection pooling** (critical)
2. **Keyring + file key storage** (essential)
3. **Basic TOML configuration** (flexibility)
4. **Access control** (security)
5. **Better error handling** (reliability)

## Architecture

```
┌─────────────┐    ┌──────────────┐    ┌─────────────┐    ┌──────────────┐
│   Browser   │───→│ http_bridge  │───→│  P2P Net    │───→│ expose_http  │
│             │    │ (local HTTP) │    │             │    │ (remote P2P) │
└─────────────┘    └──────────────┘    └─────────────┘    └──────────────┘
                           │                                       │
                           │                                       ▼
                           │                               ┌──────────────┐
                           │                               │ Local HTTP   │
                           │                               │ Service      │
                           │                               │ (with pools) │
                           │                               └──────────────┘
                           │
                   ┌──────────────┐
                   │ Connection   │
                   │ Pools        │
                   │ (bb8)        │
                   └──────────────┘
```

## Current State

- **107 lines**: expose_http.rs (basic implementation)
- **158 lines**: http_bridge.rs (basic implementation)  
- **Missing**: All production features listed above

## Next Steps

1. **Add bb8 HTTP connection pooling** - Critical for performance
2. **Implement persistent key storage** - Use malai's key management patterns
3. **Add configuration system** - Service definitions, permissions, etc.
4. **Graceful shutdown** - Proper cleanup and signal handling

malai-next must achieve feature parity with current malai while using the superior fastn-p2p API for better maintainability and performance.