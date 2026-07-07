use anyhow::{Result, anyhow};
use argon2::{Algorithm, Argon2, Params, Version};
use base64::{Engine, engine::general_purpose::STANDARD as B64};
use chacha20poly1305::{
    ChaCha20Poly1305, Nonce,
    aead::{Aead, KeyInit},
};
use rand_core::{OsRng, TryRngCore};
use zeroize::Zeroizing;

#[must_use]
pub fn generate_salt() -> String {
    let mut salt = [0u8; 32];
    OsRng.try_fill_bytes(&mut salt).expect("OS RNG failed");
    B64.encode(salt)
}

pub fn derive_key(password: &str, salt_b64: &str) -> Result<Zeroizing<[u8; 32]>> {
    let salt = B64.decode(salt_b64)?;
    let mut key = Zeroizing::new([0u8; 32]);
    let params = Params::new(65536, 4, 1, Some(32)).map_err(|e| anyhow!("Argon2 params: {e}"))?;
    Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
        .hash_password_into(password.as_bytes(), &salt, &mut *key)
        .map_err(|e| anyhow!("Key derivation failed: {e}"))?;
    Ok(key)
}

/// Decrypt ChaCha20-Poly1305 ciphertext. Used by `stash migrate` to convert
/// old field-level encrypted vaults.
pub fn decrypt(
    key: &[u8; 32],
    ciphertext: &[u8],
    nonce_bytes: &[u8],
) -> Result<Zeroizing<Vec<u8>>> {
    let cipher = ChaCha20Poly1305::new(key.into());
    let nonce: &Nonce = nonce_bytes
        .try_into()
        .map_err(|_| anyhow!("Decryption failed (wrong key or corrupted data)"))?;
    cipher
        .decrypt(nonce, ciphertext)
        .map(Zeroizing::new)
        .map_err(|_| anyhow!("Decryption failed (wrong key or corrupted data)"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encrypt(key: &[u8; 32], plaintext: &[u8]) -> Result<(Vec<u8>, Vec<u8>)> {
        let cipher = ChaCha20Poly1305::new(key.into());
        let mut nonce_bytes = [0u8; 12];
        OsRng
            .try_fill_bytes(&mut nonce_bytes)
            .expect("OS RNG failed");
        let nonce: &Nonce = (&nonce_bytes).into();
        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|_| anyhow!("Encryption failed"))?;
        Ok((ciphertext, nonce_bytes.to_vec()))
    }

    #[test]
    fn roundtrip() {
        let key = [0u8; 32];
        let plaintext = b"hello, stash!";
        let (ct, nonce) = encrypt(&key, plaintext).unwrap();
        assert_eq!(decrypt(&key, &ct, &nonce).unwrap().as_slice(), plaintext);
    }

    #[test]
    fn roundtrip_empty_plaintext() {
        let key = [1u8; 32];
        let (ct, nonce) = encrypt(&key, b"").unwrap();
        assert!(decrypt(&key, &ct, &nonce).unwrap().is_empty());
    }

    #[test]
    fn wrong_key_fails() {
        let (ct, nonce) = encrypt(&[1u8; 32], b"secret").unwrap();
        assert!(decrypt(&[2u8; 32], &ct, &nonce).is_err());
    }

    #[test]
    fn wrong_nonce_fails() {
        let key = [1u8; 32];
        let (ct, _) = encrypt(&key, b"secret").unwrap();
        assert!(decrypt(&key, &ct, &[0u8; 12]).is_err());
    }

    #[test]
    fn tampered_ciphertext_fails() {
        let key = [0u8; 32];
        let (mut ct, nonce) = encrypt(&key, b"important data").unwrap();
        ct[0] ^= 0xff;
        assert!(decrypt(&key, &ct, &nonce).is_err());
    }

    #[test]
    fn salt_is_valid_base64_and_32_bytes() {
        let s = generate_salt();
        let bytes = B64.decode(&s).unwrap();
        assert_eq!(bytes.len(), 32);
    }

    #[test]
    fn salts_are_unique() {
        assert_ne!(generate_salt(), generate_salt());
    }

    #[test]
    fn derive_key_is_deterministic() {
        let salt = generate_salt();
        assert_eq!(
            derive_key("pw", &salt).unwrap(),
            derive_key("pw", &salt).unwrap()
        );
    }

    #[test]
    fn derive_key_differs_for_different_passwords() {
        let salt = generate_salt();
        assert_ne!(
            derive_key("pw-a", &salt).unwrap(),
            derive_key("pw-b", &salt).unwrap()
        );
    }

    #[test]
    fn derive_key_differs_for_different_salts() {
        assert_ne!(
            derive_key("pw", &generate_salt()).unwrap(),
            derive_key("pw", &generate_salt()).unwrap()
        );
    }

    #[test]
    fn derive_key_rejects_invalid_base64() {
        assert!(derive_key("pw", "not!!base64").is_err());
    }
}
