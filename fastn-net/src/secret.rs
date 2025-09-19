/// Reads an existing secret key or creates a new one if none exists.
///
/// Uses fastn-id52's built-in key management:
/// 1. Try to load from current directory with "fastn" prefix
/// 2. If not found, generate new key and save it
///
/// # Errors
///
/// Returns an error if key reading/writing fails.
#[tracing::instrument]
pub async fn read_or_create_key() -> eyre::Result<(String, fastn_id52::SecretKey)> {
    let current_dir = std::path::Path::new(".");
    
    match fastn_id52::SecretKey::load_from_dir(current_dir, "fastn") {
        Ok((id52, secret_key)) => {
            tracing::info!("Loaded existing key: {id52}");
            Ok((id52, secret_key))
        }
        Err(_) => {
            tracing::info!("No existing key found, generating new one");
            let secret_key = fastn_id52::SecretKey::generate();
            let id52 = secret_key.id52();
            
            secret_key.save_to_dir(current_dir, "fastn")?;
            tracing::info!("Generated and saved new key: {id52}");
            
            Ok((id52, secret_key))
        }
    }
}
