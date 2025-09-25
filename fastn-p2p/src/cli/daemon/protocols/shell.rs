//! Shell protocol handler
//!
//! Streaming protocol for remote command execution.

use tokio::sync::broadcast;

use super::super::{DaemonResponse};

/// Shell command structure
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct ShellCommand {
    pub command: String,
    pub args: Vec<String>,
}

/// Shell response structure
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct ShellResponse {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

/// Shell error types
#[derive(Debug, thiserror::Error, serde::Serialize, serde::Deserialize)]
pub enum ShellError {
    #[error("Command execution failed: {message}")]
    ExecutionFailed { message: String },
    #[error("Command not allowed: {command}")]
    CommandNotAllowed { command: String },
    #[error("Timeout executing command")]
    Timeout,
}

/// Initialize the Shell protocol handler - creates config directory and default config
pub async fn init(
    bind_alias: String,
    config_path: std::path::PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    todo!("Create config directory, write default shell.json config file with allowed commands, security settings");
}

/// Load the Shell protocol handler - assumes config already exists
pub async fn load(
    bind_alias: String,
    config_path: std::path::PathBuf,
    identity_key: fastn_id52::SecretKey,
) -> Result<(), Box<dyn std::error::Error>> {
    todo!("Read config from config_path/shell.json, start P2P streaming listener, register shell handlers");
}

/// Reload the Shell protocol handler - re-read config and restart services
pub async fn reload(
    bind_alias: String,
    config_path: std::path::PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    todo!("Stop current shell service, re-read config, restart with updated security settings");
}

/// Stop the Shell protocol handler  
pub async fn stop(
    bind_alias: String,
) -> Result<(), Box<dyn std::error::Error>> {
    todo!("Clean shutdown of Shell protocol streaming listener and command handlers");
}

/// Check Shell protocol configuration without changing runtime
pub async fn check(
    bind_alias: String,
    config_path: std::path::PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    todo!("Validate config_path/shell.json exists, has valid security settings, allowed commands list, report issues");
}

/// Handle Shell protocol streaming sessions
pub async fn shell_stream_handler(
    mut _session: fastn_p2p::Session<&'static str>,
    command: ShellCommand,
    _state: (),
) -> Result<(), ShellError> {
    println!("🐚 Shell command requested: {} {:?}", command.command, command.args);
    
    // Basic security checks - only allow safe commands for testing
    let allowed_commands = ["echo", "whoami", "pwd", "ls", "date"];
    if !allowed_commands.contains(&command.command.as_str()) {
        return Err(ShellError::CommandNotAllowed { 
            command: command.command.clone() 
        });
    }
    
    // TODO: Execute command and stream output bidirectionally
    // For now, simulate command execution
    let simulated_output = match command.command.as_str() {
        "whoami" => "daemon_user".to_string(),
        "pwd" => "/tmp/fastn-daemon".to_string(), 
        "date" => chrono::Utc::now().to_rfc3339(),
        "echo" => command.args.join(" "),
        "ls" => "file1.txt\nfile2.txt\ndir1/".to_string(),
        _ => "Command output".to_string(),
    };
    
    println!("📤 Shell output: {}", simulated_output);
    
    // TODO: Stream the output back to client via session
    // session.copy_from(&mut simulated_output.as_bytes()).await?;
    
    Ok(())
}

/// Execute a shell command safely (for request/response mode)
pub async fn execute_command(command: ShellCommand) -> Result<ShellResponse, ShellError> {
    println!("⚡ Executing shell command: {} {:?}", command.command, command.args);
    
    // Security check
    let allowed_commands = ["echo", "whoami", "pwd", "ls", "date"];
    if !allowed_commands.contains(&command.command.as_str()) {
        return Err(ShellError::CommandNotAllowed { 
            command: command.command.clone() 
        });
    }
    
    // For testing, simulate command execution
    let (exit_code, stdout) = match command.command.as_str() {
        "whoami" => (0, "daemon_user\n".to_string()),
        "pwd" => (0, "/tmp/fastn-daemon\n".to_string()),
        "date" => (0, format!("{}\n", chrono::Utc::now().to_rfc3339())),
        "echo" => (0, format!("{}\n", command.args.join(" "))),
        "ls" => (0, "file1.txt\nfile2.txt\ndir1/\n".to_string()),
        _ => (1, "".to_string()),
    };
    
    let response = ShellResponse {
        exit_code,
        stdout,
        stderr: "".to_string(),
    };
    
    println!("✅ Shell command completed with exit code: {}", exit_code);
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_whoami_command() {
        let command = ShellCommand {
            command: "whoami".to_string(),
            args: vec![],
        };
        
        let response = execute_command(command).await.unwrap();
        assert_eq!(response.exit_code, 0);
        assert_eq!(response.stdout, "daemon_user\n");
    }
    
    #[tokio::test] 
    async fn test_disallowed_command() {
        let command = ShellCommand {
            command: "rm".to_string(),
            args: vec!["-rf".to_string(), "/".to_string()],
        };
        
        let result = execute_command(command).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ShellError::CommandNotAllowed { .. }));
    }
    
    #[tokio::test]
    async fn test_echo_command() {
        let command = ShellCommand {
            command: "echo".to_string(),
            args: vec!["Hello".to_string(), "World".to_string()],
        };
        
        let response = execute_command(command).await.unwrap();
        assert_eq!(response.exit_code, 0);
        assert_eq!(response.stdout, "Hello World\n");
    }
}