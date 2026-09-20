//! Opaque service API keys.
//!
//! A key is `mhf_` followed by 64 lowercase hex characters. Only the SHA-256
//! hex digest of the raw key is stored; the raw key is returned exactly once
//! when it is created or rotated and is never logged.

use sha2::{Digest, Sha256};
use uuid::Uuid;

/// Prefix that distinguishes a service API key from a Keycloak access token.
pub const KEY_PREFIX: &str = "mhf_";

const KEY_HEX_CHARS: usize = 64;
const KEY_BYTES: usize = KEY_HEX_CHARS / 2;

const HEX_DIGITS: &[u8; 16] = b"0123456789abcdef";

fn hex_encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(char::from(HEX_DIGITS[(byte >> 4) as usize]));
        out.push(char::from(HEX_DIGITS[(byte & 0x0f) as usize]));
    }
    out
}

/// Generate a fresh raw API key from cryptographically random UUIDv4 bytes.
#[must_use]
pub fn generate() -> String {
    let mut bytes = [0_u8; KEY_BYTES];
    let (first, second) = bytes.split_at_mut(KEY_BYTES / 2);
    first.copy_from_slice(Uuid::new_v4().as_bytes());
    second.copy_from_slice(Uuid::new_v4().as_bytes());
    format!("{KEY_PREFIX}{}", hex_encode(&bytes))
}

/// SHA-256 hex digest of a raw key, as stored at rest.
#[must_use]
pub fn hash(raw: &str) -> String {
    hex_encode(&Sha256::digest(raw.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_keys_have_the_expected_shape() {
        let key = generate();
        assert!(key.starts_with(KEY_PREFIX));
        assert_eq!(key.len(), KEY_PREFIX.len() + KEY_HEX_CHARS);
        let hex = key.strip_prefix(KEY_PREFIX).expect("prefix");
        assert!(
            hex.chars()
                .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
        );
    }

    #[test]
    fn generated_keys_are_unique() {
        assert_ne!(generate(), generate());
    }

    #[test]
    fn hashing_is_deterministic_and_fixed_length() {
        let raw = "mhf_0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let digest = hash(raw);
        assert_eq!(digest, hash(raw));
        assert_eq!(digest.len(), 64);
        assert!(digest.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn hashing_distinguishes_keys() {
        assert_ne!(hash("mhf_a"), hash("mhf_b"));
    }
}
