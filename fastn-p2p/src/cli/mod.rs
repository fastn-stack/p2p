//! CLI module for fastn-p2p daemon and client functionality

use std::path::PathBuf;

pub mod client;
pub mod daemon;
pub mod identity;

/// Get the FASTN_HOME directory from clap args, environment variable, or default
pub fn get_fastn_home(custom_home: Option<PathBuf>) -> Result<PathBuf, Box<dyn std::error::Error>> {
    if let Some(home) = custom_home {
        return Ok(home);
    }

    // Fallback to ~/.fastn if no FASTN_HOME env var or --home flag
    let home_dir = directories::UserDirs::new()
        .ok_or("Could not determine user home directory")?
        .home_dir()
        .to_path_buf();

    Ok(home_dir.join(".fastn"))
}