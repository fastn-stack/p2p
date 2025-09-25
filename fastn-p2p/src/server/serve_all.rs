//! Multi-identity server builder for modern daemon architecture
//!
//! This module provides the `serve_all()` builder that automatically discovers
//! and serves all configured identities and protocols from FASTN_HOME.

use std::path::PathBuf;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

/// Async callback type for request/response protocol commands
pub type RequestCallback = fn(
    &str,                    // identity
    &str,                    // bind_alias  
    &str,                    // protocol (e.g., "mail.fastn.com")
    &str,                    // command (e.g., "settings.add-forwarding")
    &PathBuf,               // protocol_dir
    serde_json::Value,      // request
) -> Pin<Box<dyn Future<Output = Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>>> + Send>>;

/// Async callback type for streaming protocol commands
pub type StreamCallback = fn(
    &str,                    // identity
    &str,                    // bind_alias
    &str,                    // protocol (e.g., "filetransfer.fastn.com")
    &str,                    // command (e.g., "transfer.large-file")
    &PathBuf,               // protocol_dir
    serde_json::Value,      // initial_data
) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send>>;

/// Multi-identity server builder that discovers and serves all configured protocols
pub struct ServeAllBuilder {
    fastn_home: PathBuf,
    request_callbacks: HashMap<String, RequestCallback>,  // Key: "protocol.command"
    stream_callbacks: HashMap<String, StreamCallback>,    // Key: "protocol.command"
}

impl ServeAllBuilder {
    /// Register a request/response callback for a protocol command
    ///
    /// # Arguments
    /// * `protocol` - Protocol name (e.g., "mail.fastn.com", "echo.fastn.com")
    /// * `command` - Command name (e.g., "get-mails", "send-mail", "basic-echo")
    /// * `callback` - Handler function
    ///
    /// # Example
    /// ```rust,no_run
    /// fastn_p2p::serve_all()
    ///     .handle_requests("mail.fastn.com", "get-mails", get_mails_handler)
    ///     .handle_requests("mail.fastn.com", "send-mail", send_mail_handler)
    /// ```
    pub fn handle_requests(mut self, protocol: &str, command: &str, callback: RequestCallback) -> Self {
        let key = format!("{}.{}", protocol, command);
        self.request_callbacks.insert(key, callback);
        self
    }
    
    /// Register a streaming callback for a protocol command
    ///
    /// # Arguments
    /// * `protocol` - Protocol name (e.g., "filetransfer.fastn.com") 
    /// * `command` - Command name (e.g., "large-file", "media-stream")
    /// * `callback` - Handler function
    pub fn handle_streams(mut self, protocol: &str, command: &str, callback: StreamCallback) -> Self {
        let key = format!("{}.{}", protocol, command);
        self.stream_callbacks.insert(key, callback);
        self
    }
    
    /// Start serving all configured identities and protocols
    pub async fn serve(self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🚀 Starting multi-identity P2P server");
        println!("📁 FASTN_HOME: {}", self.fastn_home.display());
        
        // Load all identity configurations using daemon utilities
        let identity_configs = super::daemon::load_all_identities(&self.fastn_home).await?;
        
        let online_identities: Vec<_> = identity_configs.into_iter()
            .filter(|id| id.online)
            .collect();
            
        if online_identities.is_empty() {
            return Err("No online identities found. Set identities online with: fastn-p2p identity-online <name>".into());
        }
        
        println!("🔑 Found {} online identities", online_identities.len());
        
        // Start P2P listeners for each identity/protocol combination
        for identity_config in online_identities {
            println!("🎧 Starting services for identity: {}", identity_config.alias);
            
            for protocol_binding in &identity_config.protocols {
                let protocol_dir = protocol_binding.config_path.clone();
                
                println!("   📡 {} {} → {}", 
                        protocol_binding.protocol, 
                        protocol_binding.bind_alias,
                        protocol_dir.display());
                
                // Check if we have a handler for this protocol
                if let Some(callback) = self.request_callbacks.get(&protocol_binding.protocol) {
                    println!("     🔄 Starting request handler for {}", protocol_binding.protocol);
                    
                    // TODO: Start actual P2P listener and route requests to callback
                    // For now, just log that we would start it
                    let identity = identity_config.alias.clone();
                    let bind_alias = protocol_binding.bind_alias.clone();
                    let protocol = protocol_binding.protocol.clone();
                    let protocol_dir_clone = protocol_dir.clone();
                    
                    tokio::spawn(async move {
                        println!("🎧 Would start P2P listener for {} {} ({})", protocol, bind_alias, identity);
                        println!("   Working dir: {}", protocol_dir_clone.display());
                        // TODO: Start fastn_p2p::listen() and route to callback
                    });
                }
                
                if let Some(callback) = self.stream_callbacks.get(&protocol_binding.protocol) {
                    println!("     🌊 Starting stream handler for {}", protocol_binding.protocol);
                    // TODO: Similar to request handler but for streaming
                }
            }
        }
        
        println!("🎯 Multi-identity server ready (TODO: implement actual P2P listening)");
        
        // Keep server running
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }
}

/// Create a new multi-identity server builder
pub fn serve_all() -> ServeAllBuilder {
    let fastn_home = std::env::var("FASTN_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or("/tmp".to_string());
            PathBuf::from(home).join(".fastn")
        });
    
    ServeAllBuilder {
        fastn_home,
        request_callbacks: HashMap::new(),
        stream_callbacks: HashMap::new(),
    }
}

/// Echo request handler callback for basic-echo command
pub fn echo_request_handler(
    identity: &str,
    bind_alias: &str,
    protocol: &str,
    command: &str,
    protocol_dir: &PathBuf,
    request: serde_json::Value,
) -> Pin<Box<dyn Future<Output = Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>>> + Send>> {
    let identity = identity.to_string();
    let bind_alias = bind_alias.to_string();
    let protocol = protocol.to_string();
    let command = command.to_string();
    let protocol_dir = protocol_dir.clone();
    
    Box::pin(async move {
        println!("💬 Echo handler called:");
        println!("   Identity: {}", identity);
        println!("   Bind alias: {}", bind_alias);
        println!("   Protocol: {}", protocol);
        println!("   Command: {}", command);
        println!("   Protocol dir: {}", protocol_dir.display());
        
        // Parse request
        let message = request.get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("(no message)");
        
        if message.is_empty() {
            return Err("Message cannot be empty".into());
        }
        
        println!("   Message: '{}'", message);
        
        // Create response
        let response = serde_json::json!({
            "echoed": format!("Echo from {} ({}): {}", identity, command, message)
        });
        
        println!("📤 Echo response: {}", response);
        Ok(response)
    })
}