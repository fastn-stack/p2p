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

/// Initialize the Shell protocol handler
pub async fn initialize(
    _daemon_key: fastn_id52::SecretKey,
    _response_tx: broadcast::Sender<DaemonResponse>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Initializing Shell protocol handler");
    
    // TODO: Set up actual P2P listener for Shell protocol
    // This would use streaming APIs for bidirectional communication
    //
    // let protocols = [TestProtocol::Shell]; 
    // fastn_p2p::listen(daemon_key)
    //     .handle_streams(TestProtocol::Shell, (), shell_stream_handler)
    //     .await?;
    
    println!("⚠️  Shell protocol handler ready (SECURITY WARNING: Remote execution enabled)");
    Ok(())
}

/// Handle Shell protocol streaming sessions
pub async fn shell_stream_handler(
    mut _session: fastn_p2p::Session,
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