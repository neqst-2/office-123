use base64::engine::general_purpose::STANDARD_NO_PAD;
use base64::Engine;
use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{ChaCha20Poly1305, Nonce};
use rand_core::RngCore;

use super::{CryptoError, KEY_LEN, NONCE_LEN};

pub fn encrypt_field(plaintext: &str, user_key: &[u8]) -> Result<String, CryptoError> {
    let key = super::derive_key(user_key);
    let cipher = ChaCha20Poly1305::new_from_slice(&key).map_err(|_| CryptoError::InvalidKey)?;

    let mut nonce_bytes = [0u8; NONCE_LEN];
    rand_core::OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|_| CryptoError::EncryptFailed)?;

    let mut out = Vec::with_capacity(NONCE_LEN + ciphertext.len());
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ciphertext);

    Ok(STANDARD_NO_PAD.encode(out))
}

pub fn decrypt_field(ciphertext_with_nonce: &str, user_key: &[u8]) -> Result<String, CryptoError> {
    let key = super::derive_key(user_key);
    let cipher = ChaCha20Poly1305::new_from_slice(&key).map_err(|_| CryptoError::InvalidKey)?;

    let decoded = STANDARD_NO_PAD
        .decode(ciphertext_with_nonce)
        .map_err(|_| CryptoError::InvalidEncoding)?;

    if decoded.len() < NONCE_LEN + 1 {
        return Err(CryptoError::InvalidCiphertext);
    }

    let nonce = Nonce::from_slice(&decoded[..NONCE_LEN]);
    let ciphertext = &decoded[NONCE_LEN..];

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| CryptoError::DecryptFailed)?;

    String::from_utf8(plaintext).map_err(|_| CryptoError::InvalidPlaintext)
}

