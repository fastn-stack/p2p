//! Pure Echo Protocol Server
//!
//! This shows how protocol developers write minimal, clean code.
//! No CLI parsing, no setup - just protocol logic and lifecycle.
//!
//! Usage:
//!   1. FASTN_HOME=/tmp/alice fastn-p2p daemon &
//!   2. FASTN_HOME=/tmp/alice fastn-p2p create-identity alice
//!   3. FASTN_HOME=/tmp/alice fastn-p2p add-protocol alice --protocol echo.fastn.com --config '{"max_length": 1000}'
//!   4. FASTN_HOME=/tmp/alice fastn-p2p identity-online alice
//!   5. FASTN_HOME=/tmp/alice cargo run --bin request_response
//!   6. echo '{"message":"Hello"}' | FASTN_HOME=/tmp/alice fastn-p2p call <alice_peer_id> echo.fastn.com basic-echo

use serde::{Serialize, Deserialize};

// Echo Protocol Types
#[derive(Serialize, Deserialize, Debug)]
pub struct EchoRequest {
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EchoResponse {
    pub echoed: String,
}

#[derive(Serialize, Deserialize, Debug, thiserror::Error)]
pub enum EchoError {
    #[error("Invalid message: {0}")]
    InvalidMessage(String),
}

#[fastn_p2p::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎧 Starting Echo protocol server");
    println!("📡 Will serve all configured echo.fastn.com protocols from FASTN_HOME");
    
    fastn_p2p::serve_all()
        .protocol("echo.fastn.com", |p| p
            .on_create(echo_create_handler)
            .on_activate(echo_activate_handler)
            .on_check(echo_check_handler)
            .handle_requests("basic-echo", fastn_p2p::echo_request_handler)
            .on_reload(echo_reload_handler)
            .on_deactivate(echo_deactivate_handler)
        )
        .serve()
        .await
}

// Lifecycle handlers - clean signatures using BindingContext
use std::pin::Pin;
use std::future::Future;

fn echo_create_handler(
    ctx: fastn_p2p::server::BindingContext,
) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send>> {
    Box::pin(async move {
        println!("🔧 Echo create: {} {} ({})", ctx.identity.id52(), ctx.bind_alias, ctx.protocol_dir.display());
        // TODO: Create default config files, setup protocol workspace
        Ok(())
    })
}

fn echo_activate_handler(
    ctx: fastn_p2p::server::BindingContext,
) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send>> {
    Box::pin(async move {
        println!("🚀 Echo activate: {} {} ({})", ctx.identity.id52(), ctx.bind_alias, ctx.protocol_dir.display());
        // TODO: Read config, start P2P listeners, initialize runtime state
        Ok(())
    })
}

fn echo_check_handler(
    ctx: fastn_p2p::server::BindingContext,
) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send>> {
    Box::pin(async move {
        println!("🔍 Echo check: {} {} ({})", ctx.identity.id52(), ctx.bind_alias, ctx.protocol_dir.display());
        // TODO: Validate config files, check runtime state
        Ok(())
    })
}

fn echo_reload_handler(
    ctx: fastn_p2p::server::BindingContext,
) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send>> {
    Box::pin(async move {
        println!("🔄 Echo reload: {} {} ({})", ctx.identity.id52(), ctx.bind_alias, ctx.protocol_dir.display());
        // TODO: Re-read config, restart services with new settings
        Ok(())
    })
}

fn echo_deactivate_handler(
    ctx: fastn_p2p::server::BindingContext,
) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send>> {
    Box::pin(async move {
        println!("🛑 Echo deactivate: {} {} ({})", ctx.identity.id52(), ctx.bind_alias, ctx.protocol_dir.display());
        // TODO: Stop accepting requests, but preserve data
        Ok(())
    })
}