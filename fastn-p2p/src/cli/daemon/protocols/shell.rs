//! Shell protocol handler
//!
//! Streaming protocol for remote command execution.

use crate::cli::daemon::protocol_trait::Protocol;

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

/// Shell protocol implementation
pub struct ShellProtocol;

#[async_trait::async_trait]
impl Protocol for ShellProtocol {
    const NAME: &'static str = "Shell";
    
    async fn init(
        bind_alias: &str,
        config_path: &std::path::PathBuf,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        todo!("Create Shell config directory, write default shell config.json with security settings for bind_alias: {}", bind_alias);
    }
    
    async fn load(
        bind_alias: &str,
        config_path: &std::path::PathBuf,
        identity_key: &fastn_id52::SecretKey,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        todo!("Load Shell config from {}, start P2P Shell streaming listener for identity {}, bind_alias: {}", config_path.display(), identity_key.public_key().id52(), bind_alias);
    }
    
    async fn reload(
        bind_alias: &str,
        config_path: &std::path::PathBuf,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        todo!("Reload Shell config from {}, restart Shell services for bind_alias: {}", config_path.display(), bind_alias);
    }
    
    async fn stop(
        bind_alias: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        todo!("Stop Shell protocol services for bind_alias: {}", bind_alias);
    }
    
    async fn check(
        bind_alias: &str,
        config_path: &std::path::PathBuf,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        todo!("Check Shell config at {} for bind_alias: {} - validate security settings, allowed commands", config_path.display(), bind_alias);
    }
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