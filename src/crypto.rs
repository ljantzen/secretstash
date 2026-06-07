use anyhow::{Result, anyhow};
use base64::{Engine, engine::general_purpose::STANDARD as B64};
use chacha20poly1305::{
    ChaCha20Poly1305, Key, Nonce,
    aead::{Aead, KeyInit},
};
use rand::{RngCore, rngs::OsRng};

pub fn generate_salt() -> String {
    let mut salt = [0u8; 32];
    OsRng.fill_bytes(&mut salt);
    B64.encode(salt)
}

pub fn derive_key(password: &str, salt_b64: &str) -> Result<[u8; 32]> {
    let salt = B64.decode(salt_b64)?;
    let mut key = [0u8; 32];
    argon2::Argon2::default()
        .hash_password_into(password.as_bytes(), &salt, &mut key)
        .map_err(|e| anyhow!("Key derivation failed: {e}"))?;
    Ok(key)
}

pub fn encrypt(key: &[u8; 32], plaintext: &[u8]) -> Result<(Vec<u8>, Vec<u8>)> {
    let cipher = ChaCha20Poly1305::new(Key::from_slice(key));
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|_| anyhow!("Encryption failed"))?;
    Ok((ciphertext, nonce_bytes.to_vec()))
}

pub fn decrypt(key: &[u8; 32], ciphertext: &[u8], nonce_bytes: &[u8]) -> Result<Vec<u8>> {
    let cipher = ChaCha20Poly1305::new(Key::from_slice(key));
    let nonce = Nonce::from_slice(nonce_bytes);
    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| anyhow!("Decryption failed (wrong key or corrupted data)"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip() {
        let key = [0u8; 32];
        let plaintext = b"hello, stash!";
        let (ct, nonce) = encrypt(&key, plaintext).unwrap();
        assert_eq!(decrypt(&key, &ct, &nonce).unwrap(), plaintext);
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
    fn nonces_are_unique() {
        let key = [0u8; 32];
        let (_, n1) = encrypt(&key, b"same").unwrap();
        let (_, n2) = encrypt(&key, b"same").unwrap();
        assert_ne!(n1, n2);
    }

    #[test]
    fn ciphertext_differs_per_call() {
        let key = [0u8; 32];
        let (ct1, _) = encrypt(&key, b"same").unwrap();
        let (ct2, _) = encrypt(&key, b"same").unwrap();
        assert_ne!(ct1, ct2);
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
