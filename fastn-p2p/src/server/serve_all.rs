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

/// Protocol binding context passed to all handlers
#[derive(Debug, Clone)]
pub struct BindingContext {
    pub identity: fastn_id52::PublicKey,
    pub bind_alias: String,
    pub protocol_dir: PathBuf,
}

/// Lifecycle callback types for protocol management (per binding) - clean async fn signatures  
pub type CreateCallback = fn(BindingContext) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send>>;
pub type ActivateCallback = fn(BindingContext) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send>>;
pub type DeactivateCallback = fn(BindingContext) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send>>;
pub type CheckCallback = fn(BindingContext) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send>>;
pub type ReloadCallback = fn(BindingContext) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send>>;
pub type DeleteCallback = fn(BindingContext) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send>>;

/// Global lifecycle callback types (across all protocol bindings)
pub type GlobalLoadCallback = fn(&str) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send>>;
pub type GlobalUnloadCallback = fn(&str) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send>>;

/// Protocol command handlers for a specific protocol
pub struct ProtocolBuilder {
    protocol_name: String,
    request_callbacks: HashMap<String, RequestCallback>,  // Key: command name
    stream_callbacks: HashMap<String, StreamCallback>,    // Key: command name
    
    // Per-binding lifecycle callbacks
    create_callback: Option<CreateCallback>,
    activate_callback: Option<ActivateCallback>,
    deactivate_callback: Option<DeactivateCallback>,
    check_callback: Option<CheckCallback>,
    reload_callback: Option<ReloadCallback>,
    delete_callback: Option<DeleteCallback>,
    
    // Global protocol lifecycle callbacks  
    global_load_callback: Option<GlobalLoadCallback>,
    global_unload_callback: Option<GlobalUnloadCallback>,
}

impl ProtocolBuilder {
    /// Add a request/response command handler (panics on duplicate)
    pub fn handle_requests(mut self, command: &str, callback: RequestCallback) -> Self {
        if self.request_callbacks.contains_key(command) {
            panic!("Duplicate request handler for protocol '{}' command '{}' - each command can only be registered once", 
                   self.protocol_name, command);
        }
        self.request_callbacks.insert(command.to_string(), callback);
        self
    }
    
    /// Add a streaming command handler (panics on duplicate)
    pub fn handle_streams(mut self, command: &str, callback: StreamCallback) -> Self {
        if self.stream_callbacks.contains_key(command) {
            panic!("Duplicate stream handler for protocol '{}' command '{}' - each command can only be registered once", 
                   self.protocol_name, command);
        }
        self.stream_callbacks.insert(command.to_string(), callback);
        self
    }
    
    /// Protocol creation (called from: fastn-p2p add-protocol)
    /// Creates workspace, default configs, initial setup
    pub fn on_create(mut self, callback: CreateCallback) -> Self {
        if self.create_callback.is_some() {
            panic!("Duplicate on_create for protocol '{}' - can only register once", self.protocol_name);
        }
        self.create_callback = Some(callback);
        self
    }
    
    /// Protocol activation (called from: fastn-p2p start, daemon startup)
    /// Start services, begin accepting requests
    pub fn on_activate(mut self, callback: ActivateCallback) -> Self {
        if self.activate_callback.is_some() {
            panic!("Duplicate on_activate for protocol '{}' - can only register once", self.protocol_name);
        }
        self.activate_callback = Some(callback);
        self
    }
    
    /// Protocol deactivation (called from: fastn-p2p stop mail default)
    /// Stop accepting requests, but preserve data
    pub fn on_deactivate(mut self, callback: DeactivateCallback) -> Self {
        if self.deactivate_callback.is_some() {
            panic!("Duplicate on_deactivate for protocol '{}' - can only register once", self.protocol_name);
        }
        self.deactivate_callback = Some(callback);
        self
    }
    
    /// Protocol configuration check (called from: fastn-p2p check)
    /// Validate configuration without affecting runtime
    pub fn on_check(mut self, callback: CheckCallback) -> Self {
        if self.check_callback.is_some() {
            panic!("Duplicate on_check for protocol '{}' - can only register once", self.protocol_name);
        }
        self.check_callback = Some(callback);
        self
    }
    
    /// Protocol reload (called from: fastn-p2p reload mail default)
    /// Re-read config, restart services with new settings
    pub fn on_reload(mut self, callback: ReloadCallback) -> Self {
        if self.reload_callback.is_some() {
            panic!("Duplicate on_reload for protocol '{}' - can only register once", self.protocol_name);
        }
        self.reload_callback = Some(callback);
        self
    }
    
    /// Protocol deletion (called from: fastn-p2p delete mail default)
    /// Complete cleanup, remove all data and configs
    pub fn on_delete(mut self, callback: DeleteCallback) -> Self {
        if self.delete_callback.is_some() {
            panic!("Duplicate on_delete for protocol '{}' - can only register once", self.protocol_name);
        }
        self.delete_callback = Some(callback);
        self
    }
    
    /// Global protocol load (once per protocol, across all bindings)
    pub fn on_global_load(mut self, callback: GlobalLoadCallback) -> Self {
        if self.global_load_callback.is_some() {
            panic!("Duplicate on_global_load for protocol '{}' - can only register once", self.protocol_name);
        }
        self.global_load_callback = Some(callback);
        self
    }
    
    /// Global protocol unload (once per protocol, across all bindings)  
    pub fn on_global_unload(mut self, callback: GlobalUnloadCallback) -> Self {
        if self.global_unload_callback.is_some() {
            panic!("Duplicate on_global_unload for protocol '{}' - can only register once", self.protocol_name);
        }
        self.global_unload_callback = Some(callback);
        self
    }
}

/// Multi-identity server builder that discovers and serves all configured protocols
pub struct ServeAllBuilder {
    fastn_home: PathBuf,
    protocols: HashMap<String, ProtocolBuilder>,  // Key: protocol name
}

impl ServeAllBuilder {
    /// Register handlers for a protocol with nested command structure
    ///
    /// # Example
    /// ```rust,no_run
    /// fastn_p2p::serve_all()
    ///     .protocol("mail.fastn.com", |p| p
    ///         .handle_requests("get-mails", get_mails_handler)
    ///         .handle_requests("send-mail", send_mail_handler)
    ///         .handle_requests("settings.add-forwarding", forwarding_handler)
    ///     )
    ///     .protocol("filetransfer.fastn.com", |p| p
    ///         .handle_streams("transfer.large-file", large_file_handler)
    ///     )
    /// ```
    /// Register a protocol with its commands and lifecycle (panics on duplicate)
    pub fn protocol<F>(mut self, protocol_name: &str, builder_fn: F) -> Self 
    where
        F: FnOnce(ProtocolBuilder) -> ProtocolBuilder,
    {
        if self.protocols.contains_key(protocol_name) {
            panic!("Duplicate protocol registration for '{}' - each protocol can only be registered once", protocol_name);
        }
        
        let protocol_builder = ProtocolBuilder {
            protocol_name: protocol_name.to_string(),
            request_callbacks: HashMap::new(),
            stream_callbacks: HashMap::new(),
            create_callback: None,
            activate_callback: None,
            deactivate_callback: None,
            check_callback: None,
            reload_callback: None,
            delete_callback: None,
            global_load_callback: None,
            global_unload_callback: None,
        };
        
        let configured_protocol = builder_fn(protocol_builder);
        self.protocols.insert(protocol_name.to_string(), configured_protocol);
        self
    }
    
    /// Start serving all configured identities and protocols
    pub async fn serve(self) -> Result<(), Box<dyn std::error::Error>> {
        // Magic CLI detection - check if args look like CLI commands
        let args: Vec<String> = std::env::args().collect();
        if args.len() > 1 {
            match args[1].as_str() {
                "init" => {
                    return self.handle_init_command().await;
                },
                "call" => {
                    return self.handle_call_command(args).await;
                },
                "stream" => {
                    return self.handle_stream_command(args).await;
                },
                "create-identity" => {
                    return self.handle_create_identity_command(args).await;
                },
                "add-protocol" => {
                    return self.handle_add_protocol_command(args).await;
                },
                "remove-protocol" => {
                    return self.handle_remove_protocol_command(args).await;
                },
                "status" => {
                    return self.handle_status_command().await;
                },
                "identity-online" => {
                    return self.handle_identity_online_command(args).await;
                },
                "identity-offline" => {
                    return self.handle_identity_offline_command(args).await;
                },
                "run" => {
                    // Continue to server mode
                },
                _ => {
                    // If not recognized as CLI command, continue to server mode
                }
            }
        }

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
                if let Some(protocol_builder) = self.protocols.get(&protocol_binding.protocol) {
                    if !protocol_builder.request_callbacks.is_empty() {
                        println!("     🔄 Starting request handlers for {}", protocol_binding.protocol);
                        
                        // TODO: Start actual P2P listener and route requests to callbacks
                        // For now, just log that we would start it
                        let identity = identity_config.alias.clone();
                        let bind_alias = protocol_binding.bind_alias.clone();
                        let protocol = protocol_binding.protocol.clone();
                        let protocol_dir_clone = protocol_dir.clone();
                        
                        tokio::spawn(async move {
                            println!("🎧 Would start P2P listener for {} {} ({})", protocol, bind_alias, identity);
                            println!("   Working dir: {}", protocol_dir_clone.display());
                            // TODO: Start fastn_p2p::listen() and route to callbacks
                        });
                    }
                    
                    if !protocol_builder.stream_callbacks.is_empty() {
                        println!("     🌊 Starting stream handlers for {}", protocol_binding.protocol);
                        // TODO: Similar to request handler but for streaming
                    }
                }
            }
        }
        
        println!("🎯 Multi-identity server ready (TODO: implement actual P2P listening)");
        
        // Keep server running
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }

    // Magic CLI command handlers
    async fn handle_init_command(&self) -> Result<(), Box<dyn std::error::Error>> {
        crate::cli::init::run(self.fastn_home.clone()).await
    }

    async fn handle_call_command(&self, args: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
        if args.len() < 4 {
            eprintln!("Usage: {} call <peer> <protocol> [bind_alias] [--as-identity <identity>]", args[0]);
            std::process::exit(1);
        }
        let peer = args[2].clone();
        let protocol = args[3].clone();
        let bind_alias = args.get(4).cloned().unwrap_or_else(|| "default".to_string());
        let as_identity = None; // TODO: parse --as-identity flag
        
        crate::cli::client::call(self.fastn_home.clone(), peer, protocol, bind_alias, as_identity).await
    }

    async fn handle_stream_command(&self, args: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
        if args.len() < 4 {
            eprintln!("Usage: {} stream <peer> <protocol>", args[0]);
            std::process::exit(1);
        }
        let peer = args[2].clone();
        let protocol = args[3].clone();
        
        crate::cli::client::stream(self.fastn_home.clone(), peer, protocol).await
    }

    async fn handle_create_identity_command(&self, args: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
        if args.len() < 3 {
            eprintln!("Usage: {} create-identity <alias>", args[0]);
            std::process::exit(1);
        }
        let alias = args[2].clone();
        
        crate::cli::identity::create_identity(self.fastn_home.clone(), alias).await
    }

    async fn handle_add_protocol_command(&self, args: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
        // Parse: add-protocol alice --protocol Echo --alias default --config '{...}'
        let mut identity = None;
        let mut protocol = None;
        let mut alias = "default".to_string();
        let mut config = "{}".to_string();

        let mut i = 2;
        while i < args.len() {
            match args[i].as_str() {
                "--protocol" => {
                    if i + 1 < args.len() {
                        protocol = Some(args[i + 1].clone());
                        i += 2;
                    } else {
                        eprintln!("--protocol requires a value");
                        std::process::exit(1);
                    }
                },
                "--alias" => {
                    if i + 1 < args.len() {
                        alias = args[i + 1].clone();
                        i += 2;
                    } else {
                        eprintln!("--alias requires a value");
                        std::process::exit(1);
                    }
                },
                "--config" => {
                    if i + 1 < args.len() {
                        config = args[i + 1].clone();
                        i += 2;
                    } else {
                        eprintln!("--config requires a value");
                        std::process::exit(1);
                    }
                },
                _ => {
                    if identity.is_none() {
                        identity = Some(args[i].clone());
                    }
                    i += 1;
                }
            }
        }

        let identity = identity.ok_or("Missing identity argument")?;
        let protocol = protocol.ok_or("Missing --protocol argument")?;

        crate::cli::identity::add_protocol(self.fastn_home.clone(), identity, protocol, alias, config).await
    }

    async fn handle_remove_protocol_command(&self, args: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
        // Parse similar to add_protocol
        let mut identity = None;
        let mut protocol = None;
        let mut alias = "default".to_string();

        let mut i = 2;
        while i < args.len() {
            match args[i].as_str() {
                "--protocol" => {
                    if i + 1 < args.len() {
                        protocol = Some(args[i + 1].clone());
                        i += 2;
                    } else {
                        eprintln!("--protocol requires a value");
                        std::process::exit(1);
                    }
                },
                "--alias" => {
                    if i + 1 < args.len() {
                        alias = args[i + 1].clone();
                        i += 2;
                    } else {
                        eprintln!("--alias requires a value");
                        std::process::exit(1);
                    }
                },
                _ => {
                    if identity.is_none() {
                        identity = Some(args[i].clone());
                    }
                    i += 1;
                }
            }
        }

        let identity = identity.ok_or("Missing identity argument")?;
        let protocol = protocol.ok_or("Missing --protocol argument")?;

        crate::cli::identity::remove_protocol(self.fastn_home.clone(), identity, protocol, alias).await
    }

    async fn handle_status_command(&self) -> Result<(), Box<dyn std::error::Error>> {
        crate::cli::status::show_status(self.fastn_home.clone()).await
    }

    async fn handle_identity_online_command(&self, args: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
        if args.len() < 3 {
            eprintln!("Usage: {} identity-online <identity>", args[0]);
            std::process::exit(1);
        }
        let identity = args[2].clone();
        
        crate::cli::identity::set_identity_online(self.fastn_home.clone(), identity).await
    }

    async fn handle_identity_offline_command(&self, args: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
        if args.len() < 3 {
            eprintln!("Usage: {} identity-offline <identity>", args[0]);
            std::process::exit(1);
        }
        let identity = args[2].clone();
        
        crate::cli::identity::set_identity_offline(self.fastn_home.clone(), identity).await
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
        protocols: HashMap::new(),
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