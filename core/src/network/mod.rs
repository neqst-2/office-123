use std::{
    net::{IpAddr, SocketAddr},
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use axum::http::HeaderMap;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use ed25519_dalek::{Signature, Signer, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::fs;
use tokio::io::AsyncWriteExt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClientMode {
    Local,
    Remote,
}

#[derive(Clone, Debug, Serialize)]
pub struct PeerInfo {
    pub addr: String,
    pub mode: ClientMode,
    pub connected_at_unix: u64,
}

#[derive(Clone)]
pub struct RemoteClientGuard {
    expected_remote_token: Option<String>,
    master_pubkey: Option<VerifyingKey>,
    forensic_dir: PathBuf,
}

impl RemoteClientGuard {
    pub fn from_env() -> Self {
        let expected_remote_token = std::env::var("NEQST_REMOTE_TOKEN").ok().filter(|v| !v.is_empty());
        let master_pubkey = std::env::var("NEQST_MASTER_PUBKEY_ED25519_B64")
            .ok()
            .and_then(|v| parse_ed25519_pubkey_b64(&v).ok());

        let forensic_dir = std::env::var("NEQST_FORENSIC_DIR")
            .ok()
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("../ForensicData"));

        Self {
            expected_remote_token,
            master_pubkey,
            forensic_dir,
        }
    }

    pub async fn authorize_ws(
        &self,
        peer: SocketAddr,
        headers: &HeaderMap,
        token_param: Option<&str>,
    ) -> Result<ClientMode, GuardError> {
        let ip = peer.ip();
        if is_loopback(ip) {
            return Ok(ClientMode::Local);
        }

        let token = token_param
            .map(str::to_string)
            .or_else(|| headers.get("x-neqst-remote-token").and_then(|v| v.to_str().ok()).map(str::to_string));

        let Some(token) = token else {
            self.forensic_flag(
                "remote_auth_missing_token",
                peer,
                json!({"reason": "missing_token"}),
            )
            .await;
            return Err(GuardError::MissingToken);
        };

        if let Some(expected) = self.expected_remote_token.as_ref() {
            if &token != expected {
                self.forensic_flag(
                    "remote_auth_token_mismatch",
                    peer,
                    json!({"reason": "token_mismatch"}),
                )
                .await;
                return Err(GuardError::TokenMismatch);
            }
        }

        let Some(pubkey) = self.master_pubkey.as_ref() else {
            self.forensic_flag(
                "remote_auth_pubkey_unavailable",
                peer,
                json!({"reason": "pubkey_unavailable"}),
            )
            .await;
            return Err(GuardError::KeyUnavailable);
        };

        if let Err(err) = verify_paseto_v4_public(&token, pubkey) {
            self.forensic_flag(
                "remote_auth_signature_invalid",
                peer,
                json!({"reason": "signature_invalid", "detail": err}),
            )
            .await;
            return Err(GuardError::InvalidSignature);
        }

        Ok(ClientMode::Remote)
    }

    pub async fn forensic_flag(&self, kind: &str, peer: SocketAddr, data: serde_json::Value) {
        let ts = unix_now();
        let event = json!({
            "ts_unix": ts,
            "kind": kind,
            "peer": peer.to_string(),
            "data": data
        });

        let _ = fs::create_dir_all(&self.forensic_dir).await;
        let path = self.forensic_dir.join("neqst_forensics.ndjson");
        if let Ok(mut f) = fs::OpenOptions::new().create(true).append(true).open(path).await {
            let _ = f.write_all(event.to_string().as_bytes()).await;
            let _ = f.write_all(b"\n").await;
        }
    }
}

#[derive(Debug)]
pub enum GuardError {
    MissingToken,
    TokenMismatch,
    KeyUnavailable,
    InvalidSignature,
}

#[derive(Debug, Deserialize)]
struct PasetoClaims {
    exp: u64,
    iat: u64,
    aud: String,
    nonce: String,
}

pub fn sign_paseto_v4_public(
    payload_bytes: &[u8],
    signing_key: &ed25519_dalek::SigningKey,
) -> String {
    let pae = pae(&[b"v4.public.", payload_bytes, b"", b""]);
    let sig = signing_key.sign(&pae);
    format!(
        "v4.public.{}.{}",
        URL_SAFE_NO_PAD.encode(payload_bytes),
        URL_SAFE_NO_PAD.encode(sig.to_bytes())
    )
}

fn verify_paseto_v4_public(token: &str, pubkey: &VerifyingKey) -> Result<(), String> {
    let Some(rest) = token.strip_prefix("v4.public.") else {
        return Err("invalid_paseto_prefix".to_string());
    };

    let mut parts = rest.split('.');
    let Some(payload_b64) = parts.next() else { return Err("invalid_payload".to_string()) };
    let Some(sig_b64) = parts.next() else { return Err("invalid_sig".to_string()) };
    if parts.next().is_some() {
        return Err("footer_not_supported".to_string());
    }

    let payload_bytes = URL_SAFE_NO_PAD
        .decode(payload_b64)
        .map_err(|_| "payload_b64_decode_failed".to_string())?;
    let sig_bytes = URL_SAFE_NO_PAD
        .decode(sig_b64)
        .map_err(|_| "sig_b64_decode_failed".to_string())?;

    let claims: PasetoClaims =
        serde_json::from_slice(&payload_bytes).map_err(|_| "payload_json_invalid".to_string())?;

    if claims.aud != "neqst-remote" {
        return Err("aud_mismatch".to_string());
    }
    let now = unix_now();
    if claims.exp <= now {
        return Err("token_expired".to_string());
    }
    if claims.iat > now + 30 {
        return Err("iat_in_future".to_string());
    }

    let sig = Signature::from_slice(&sig_bytes).map_err(|_| "sig_invalid".to_string())?;
    let pae = pae(&[b"v4.public.", payload_bytes.as_slice(), b"", b""]);
    pubkey
        .verify_strict(&pae, &sig)
        .map_err(|_| "signature_verify_failed".to_string())?;

    Ok(())
}

fn pae(pieces: &[&[u8]]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&(pieces.len() as u64).to_le_bytes());
    for p in pieces {
        out.extend_from_slice(&(p.len() as u64).to_le_bytes());
        out.extend_from_slice(p);
    }
    out
}

fn parse_ed25519_pubkey_b64(value: &str) -> Result<VerifyingKey, String> {
    let bytes = URL_SAFE_NO_PAD
        .decode(value)
        .or_else(|_| base64::engine::general_purpose::STANDARD_NO_PAD.decode(value))
        .map_err(|_| "pubkey_b64_decode_failed".to_string())?;

    let key_bytes: [u8; 32] = bytes
        .as_slice()
        .try_into()
        .map_err(|_| "pubkey_wrong_length".to_string())?;
    VerifyingKey::from_bytes(&key_bytes).map_err(|_| "pubkey_invalid".to_string())
}

fn is_loopback(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => v4.is_loopback(),
        IpAddr::V6(v6) => v6.is_loopback(),
    }
}

pub fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

