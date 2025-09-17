# Identity and Key Management

fastn-p2p uses Ed25519 cryptographic keys for peer identity and authentication.

## What is ID52?

ID52 is a peer identifier - a 52-character string using BASE32_DNSSEC encoding that uniquely identifies each peer. The format is:
- Exactly 52 characters long
- Uses only lowercase letters and digits
- DNS-compatible (can be used in subdomains)
- URL-safe without special encoding

## Key Types

- **`SecretKey`**: Private key for signing operations and peer identity
- **`PublicKey`**: Public key with ID52 encoding for peer identification
- **`Signature`**: Ed25519 signature for message authentication

## Basic Usage

```rust
use fastn_p2p::{SecretKey, PublicKey};

// Generate a new peer identity
let secret_key = SecretKey::generate();
let public_key = secret_key.public_key();

// Get the peer's ID52 identifier
let peer_id = public_key.to_string(); // 52-character ID52
assert_eq!(peer_id.len(), 52);

// Sign and verify a message
let message = b"Hello, P2P!";
let signature = secret_key.sign(message);
assert!(public_key.verify(message, &signature).is_ok());
```

## Key Loading

The library provides comprehensive key loading with automatic fallback:

```rust
use fastn_p2p::SecretKey;
use std::path::Path;

// Load from directory (checks for .id52 or .private-key files)
let (id52, key) = SecretKey::load_from_dir(Path::new("/path"), "peer")?;

// Load with fallback: keyring → FASTN_SECRET_KEYS_FILE → FASTN_SECRET_KEYS
let key = SecretKey::load_for_id52("i66fo538...")?;
```

## Environment Variables

- **`FASTN_SECRET_KEYS`**: Keys directly in env var (format: `prefix: hexkey`)
- **`FASTN_SECRET_KEYS_FILE`**: Path to file containing keys (more secure)

Cannot have both set (strict mode). Files support comments (`#`) and empty lines.

## CLI Tool

The `fastn-p2p-keygen` CLI tool generates peer identities:

```bash
# Default: Generate and store in system keyring
fastn-p2p-keygen generate
# Output: ID52 printed to stdout, secret key stored in keyring

# Save to file (less secure, requires explicit flag)
fastn-p2p-keygen generate --file my-peer.key

# Print to stdout
fastn-p2p-keygen generate --file -

# Short output (only ID52, no descriptive messages)
fastn-p2p-keygen generate --short
```

By default, secret keys are stored securely in the system keyring and can be viewed in your password manager.

## DNS Resolution (Optional)

With the `dns` feature enabled, you can resolve public keys from DNS TXT records:

```bash
fastn-p2p-keygen resolve example.com alice
```

This looks for a TXT record: `"alice=<52-char-public-key>"`

## Error Types

- **`ParseId52Error`**: Errors when parsing ID52 strings
- **`InvalidKeyBytesError`**: Invalid key byte format
- **`ParseSecretKeyError`**: Errors parsing secret key strings
- **`InvalidSignatureBytesError`**: Invalid signature byte format
- **`SignatureVerificationError`**: Signature verification failures
- **`KeyringError`**: Errors when accessing the system keyring

## Security

This module uses `ed25519-dalek` for all cryptographic operations, which provides constant-time implementations to prevent timing attacks. Random key generation uses the operating system's secure random number generator.