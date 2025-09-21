//! Simple Remote Shell Example
//!
//! Remote command execution with merged stdout/stderr.
//! Like SSH but over P2P, simplified version.
//!
//! This is the basic remote shell - stdout and stderr are merged.
//!
//! Usage:
//!   shell_simple daemon [key]              # Start shell daemon
//!   shell_simple exec <id52> <command>     # Execute command

use serde::{Deserialize, Serialize};

// Protocol for shell commands
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ShellProtocol {
    Execute,
}

// Simple command structure  
#[derive(Debug, Serialize, Deserialize)]
pub struct ShellCommand {
    pub cmd: String,
    pub args: Vec<String>,
}

// Shell error type
#[derive(Debug, thiserror::Error)]
pub enum ShellError {
    #[error("Command failed: {0}")]
    CommandFailed(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[fastn_p2p::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage:");
        eprintln!("  {} daemon [key]           # Start shell daemon", args[0]);
        eprintln!("  {} exec <id52> <command>  # Execute command", args[0]);
        return Ok(());
    }
    
    match args[1].as_str() {
        "daemon" => {
            let private_key = if args.len() > 2 {
                args[2].parse()?
            } else {
                fastn_p2p::SecretKey::generate()
            };
            run_daemon(private_key).await
        }
        "exec" => {
            if args.len() < 4 {
                eprintln!("Usage: {} exec <id52> <command> [args...]", args[0]);
                return Ok(());
            }
            let target: fastn_p2p::PublicKey = args[2].parse()?;
            let cmd = args[3].clone();
            let cmd_args = args[4..].to_vec();
            run_client(target, cmd, cmd_args).await
        }
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            Ok(())
        }
    }
}

async fn run_daemon(private_key: fastn_p2p::SecretKey) -> Result<(), Box<dyn std::error::Error>> {
    println!("🐚 Shell daemon listening on: {}", private_key.id52());
    println!("⚠️  WARNING: This allows remote command execution!");
    println!("");
    println!("🚀 To execute commands from another machine, run:");
    println!("   cargo run --bin shell_simple -- exec {} <command>", private_key.id52());
    println!("   Example: cargo run --bin shell_simple -- exec {} \"ls -la\"", private_key.id52());
    println!("");
    
    fastn_p2p::listen(private_key)
        .handle_streams(ShellProtocol::Execute, (), shell_handler)
        .await?;
    
    Ok(())
}

async fn run_client(
    target: fastn_p2p::PublicKey,
    cmd: String,
    args: Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let private_key = fastn_p2p::SecretKey::generate();
    
    println!("📤 Executing: {} {}", cmd, args.join(" "));
    
    let command = ShellCommand { cmd, args };
    
    let mut session = fastn_p2p::client::connect(
        private_key,
        target,
        ShellProtocol::Execute,
        command,
    ).await?;
    
    // Stream output to stdout
    tokio::io::copy(&mut session.stdout, &mut tokio::io::stdout()).await?;
    
    Ok(())
}

async fn shell_handler(
    mut session: fastn_p2p::Session<ShellProtocol>,
    command: ShellCommand,
    _state: (),
) -> Result<(), ShellError> {
    println!("🔧 Executing: {} {:?} for {}", 
             command.cmd, command.args, session.peer().id52());
    
    // Execute command with merged stdout/stderr
    let output = tokio::process::Command::new(&command.cmd)
        .args(&command.args)
        .output()
        .await?;
    
    // Send both stdout and stderr (merged) 
    use tokio::io::AsyncWriteExt;
    session.send.write_all(&output.stdout).await
        .map_err(|e| ShellError::Io(e.into()))?;
    session.send.write_all(&output.stderr).await
        .map_err(|e| ShellError::Io(e.into()))?;
    
    if !output.status.success() {
        let msg = format!("\nCommand failed with exit code: {:?}", output.status.code());
        session.send.write_all(msg.as_bytes()).await
            .map_err(|e| ShellError::Io(e.into()))?;
    }
    
    Ok(())
}