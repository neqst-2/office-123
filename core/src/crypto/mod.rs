use base64::engine::general_purpose::STANDARD_NO_PAD;
use base64::Engine;

pub mod aead;

pub const NONCE_LEN: usize = 12;
pub const KEY_LEN: usize = 32;

#[derive(Debug)]
pub enum CryptoError {
    KeyUnavailable,
    InvalidKey,
    InvalidEncoding,
    InvalidCiphertext,
    InvalidPlaintext,
    EncryptFailed,
    DecryptFailed,
}

pub struct CryptoEngine {
    user_key: Option<[u8; KEY_LEN]>,
}

impl CryptoEngine {
    pub fn from_env() -> Result<Self, CryptoError> {
        let key_b64 = std::env::var("NEQST_USER_KEY_B64").ok().filter(|v| !v.is_empty());
        let key = match key_b64 {
            Some(v) => parse_key_b64(&v)?,
            None => None,
        };

        Ok(Self { user_key: key })
    }

    pub fn has_user_key(&self) -> bool {
        self.user_key.is_some()
    }

    pub fn encrypt_field(&self, plaintext: &str) -> Result<String, CryptoError> {
        let key = self.user_key.ok_or(CryptoError::KeyUnavailable)?;
        aead::encrypt_field(plaintext, &key)
    }

    pub fn decrypt_field(&self, ciphertext_with_nonce: &str) -> Result<String, CryptoError> {
        let key = self.user_key.ok_or(CryptoError::KeyUnavailable)?;
        aead::decrypt_field(ciphertext_with_nonce, &key)
    }

    pub fn encrypt_field_with_key(plaintext: &str, user_key: &[u8]) -> Result<String, CryptoError> {
        aead::encrypt_field(plaintext, user_key)
    }

    pub fn decrypt_field_with_key(
        ciphertext_with_nonce: &str,
        user_key: &[u8],
    ) -> Result<String, CryptoError> {
        aead::decrypt_field(ciphertext_with_nonce, user_key)
    }
}

impl Default for CryptoEngine {
    fn default() -> Self {
        Self { user_key: None }
    }
}

fn parse_key_b64(value: &str) -> Result<Option<[u8; KEY_LEN]>, CryptoError> {
    let decoded = STANDARD_NO_PAD.decode(value).map_err(|_| CryptoError::InvalidEncoding)?;
    if decoded.is_empty() {
        return Ok(None);
    }
    Ok(Some(derive_key(&decoded)))
}

fn derive_key(user_key: &[u8]) -> [u8; KEY_LEN] {
    if user_key.len() == KEY_LEN {
        let mut out = [0u8; KEY_LEN];
        out.copy_from_slice(user_key);
        return out;
    }
    *blake3::hash(user_key).as_bytes()
}

/*
# POST-QUANTUM ENVELOPE DESIGN

NeQST stores only ciphertext in SurrealDB. Symmetric field encryption uses an AEAD (currently
ChaCha20-Poly1305). The symmetric Data Encryption Key (DEK) is never stored in the database.

Hybrid envelope strategy (PQC-ready):

1) Generate a fresh per-record DEK (32 bytes) and a per-field nonce (12 bytes).
2) Encrypt fields with AEAD(DEK, nonce, aad=scoped metadata).
3) Wrap the DEK for each recipient principal using a hybrid KEM:
   - ML-KEM (Kyber1024) encapsulation to recipient PQ public key → (ct_pq, ss_pq)
   - X25519 encapsulation to recipient classical public key → (ct_ec, ss_ec)
   - Combine shared secrets: ss = KDF(ss_pq || ss_ec || context)
4) Encrypt(DEK) under ss using an AEAD key wrap, store envelope metadata:
   { alg: "hybrid-mlkem1024+x25519", recipient_key_id: "...", ct_pq: "...", ct_ec: "...", wrapped_dek: "..." }
5) Decryption occurs only inside the Rust Security Layer, using OS keystore-backed private keys.

For this phase, the code implements the AEAD layer and a single in-memory key derived from
NEQST_USER_KEY_B64. The hybrid KEM wrapping is specified here and will be implemented once
the selected PQC crate (ML-KEM) and keystore strategy are finalized.
*/

