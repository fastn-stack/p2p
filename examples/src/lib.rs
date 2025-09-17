//! Shared utilities for P2P examples

/// Parse a key from string or generate a new one
pub fn key_from_str_or_generate(key_str: Option<&str>) -> Result<fastn_id52::SecretKey, Box<dyn std::error::Error>> {
    match key_str {
        Some(s) => Ok(s.parse()?),
        None => Ok(fastn_id52::SecretKey::generate()),
    }
}

/// Parse a peer ID from string
pub fn parse_peer_id(id52_str: &str) -> Result<fastn_id52::PublicKey, Box<dyn std::error::Error>> {
    Ok(id52_str.parse()?)
}