//! Client functionality for fastn-p2p CLI
//!
//! This module handles CLI client commands using the lightweight fastn-p2p-client crate.
//! It provides the same functionality as the examples but via CLI interface.

use std::path::PathBuf;
use std::io::{self, Read};

/// Make a request/response call to a peer via the daemon
pub async fn call(
    _fastn_home: PathBuf,
    peer_id52: String,
    protocol: String,
    bind_alias: String,
    as_identity: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Determine identity to send from
    let from_identity = match as_identity {
        Some(identity) => identity,
        None => {
            // TODO: Auto-detect identity if only one configured
            "alice".to_string() // Hardcoded for testing
        }
    };
    
    // Read JSON request from stdin
    let mut stdin_input = String::new();
    io::stdin().read_to_string(&mut stdin_input)?;
    let stdin_input = stdin_input.trim();
    
    if stdin_input.is_empty() {
        return Err("No JSON input provided on stdin".into());
    }
    
    // Parse JSON to validate it's valid
    let request_json: serde_json::Value = serde_json::from_str(stdin_input)?;
    
    println!("📤 Sending {} {} request from {} to {}", protocol, bind_alias, from_identity, peer_id52);
    
    // For now, demonstrate with hardcoded protocol types to test coordination
    // TODO: Make this generic once daemon coordination is working
    if protocol == "Echo" {
        // Create EchoRequest from JSON input
        let echo_request: crate::cli::daemon::test_protocols::EchoRequest = 
            serde_json::from_value(request_json)?;
        
        let result: crate::cli::daemon::test_protocols::EchoResult = 
            fastn_p2p_client::call(&from_identity, &peer_id52, &protocol, &bind_alias, echo_request).await?;
        
        // Print result as JSON
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        return Err(format!("Protocol '{}' not supported in CLI yet", protocol).into());
    }
    
    Ok(())
}

/// Open a bidirectional stream to a peer via the daemon  
pub async fn stream(
    _fastn_home: PathBuf,
    _peer: String,
    _protocol: String,
) -> Result<(), Box<dyn std::error::Error>> {
    todo!("Use fastn-p2p-client::connect() to establish stream via daemon, pipe stdin/stdout bidirectionally");
}