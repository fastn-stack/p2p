# malai-next: Production-Grade HTTP over P2P

Next-generation malai implementation using the new fastn-p2p API. This is a fresh implementation that fixes bugs and performance issues in the legacy malai HTTP proxy functionality.

## Overview

malai-next provides production-grade HTTP over P2P with:
- **expose_http**: P2P server that exposes local HTTP services
- **http_bridge**: P2P client that creates local HTTP→P2P bridges

## Essential Features Analysis

Based on analysis of the current malai implementation, the following production features are essential:

### ✅ Currently Implemented
1. **Basic HTTP Proxying** - Bidirectional HTTP forwarding over P2P
2. **Custom Error Types** - HttpError, BridgeError with specific variants
3. **Efficient Streaming** - session.copy_both() for zero-copy bidirectional I/O
4. **Clean API** - Uses new fastn_p2p::listen().handle_streams() pattern

### 🚧 Missing Critical Features (TODO)
1. **HTTP Connection Pooling** - bb8::Pool for connection reuse (performance critical)
2. **Graceful Shutdown** - Proper signal handling and cleanup
3. **Key Management** - Persistent key storage/retrieval (not generate each time)
4. **Configuration Management** - Config files for services, permissions, etc.
5. **Permission Control** - Access control for who can proxy what services
6. **Service Discovery** - Automatic discovery of available services
7. **Health Checks** - Monitor upstream service availability
8. **Metrics/Logging** - Proper observability for production use
9. **Multiple Services** - Single server exposing multiple HTTP services
10. **DNS Integration** - Domain-based addressing like original malai

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

1. **❌ No connection pooling** - Creates new TCP connection per request
2. **❌ No graceful shutdown** - Basic tokio::select only
3. **❌ No configuration** - Hardcoded host/port, no permissions
4. **❌ No persistent keys** - Generates new key each run
5. **❌ No service management** - Single service only
6. **❌ No access control** - Anyone can connect
7. **❌ No proper error context** - Basic error types only

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

**Remaining critical gaps before production readiness:**
1. **HTTP connection pooling** (performance)
2. **Persistent key management** (functionality)  
3. **Configuration system** (flexibility)
4. **Access control** (security)

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