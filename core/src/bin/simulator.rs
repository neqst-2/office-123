use std::net::SocketAddr;

use serde_json::json;

use neqst_core::crypto::CryptoEngine;
use neqst_core::network::RemoteClientGuard;
use neqst_core::orchestrator::{self, RuntimeConfig};
use neqst_core::{lok, queue, rpc, storage};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    std::env::set_var("NEQST_DB_ENGINE", "mem");
    std::env::set_var("NEQST_FORENSIC_DIR", "../ForensicData");

    let config = RuntimeConfig::from_env();
    let db = storage::init_db(&config).await?;
    storage::apply_schema(&db).await?;

    crypto_layer_verification(&db).await?;
    libreofficekit_mock_execution(&db).await?;
    p2p_guard_check().await?;
    worker_queue_check().await?;
    bootstrap_node_status_check().await?;

    println!("SIMULATOR_OK");
    Ok(())
}

async fn crypto_layer_verification(
    db: &neqst_core::orchestrator::Db,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let key = [7u8; 32];
    let plaintext = "hello neqst crypto";
    let ciphertext = CryptoEngine::encrypt_field_with_key(plaintext, &key)?;
    let decrypted = CryptoEngine::decrypt_field_with_key(&ciphertext, &key)?;
    if decrypted != plaintext {
        return Err("crypto_roundtrip_failed".into());
    }

    let enc_meta = json!({"e2ee": true, "alg": "chacha20poly1305", "encoding": "b64", "v": 1});
    let sql = r#"
UPSERT mail SET
  message_id = $message_id,
  subject = $subject,
  body_text = $body_text,
  body_html = $body_html,
  date_received = time::now(),
  is_read = false,
  size = 123,
  encryption_metadata = $encryption_metadata
WHERE message_id = $message_id;
SELECT body_text, encryption_metadata FROM mail WHERE message_id = $message_id LIMIT 1;
"#;

    let mut res = db
        .query_bind(
            sql,
            vec![
                ("message_id".to_string(), surrealdb::sql::Value::from("sim-msg-1")),
                ("subject".to_string(), surrealdb::sql::Value::from("sim subject")),
                ("body_text".to_string(), surrealdb::sql::Value::from(ciphertext.clone())),
                ("body_html".to_string(), surrealdb::sql::Value::from(ciphertext.clone())),
                ("encryption_metadata".to_string(), surrealdb::sql::Value::from(enc_meta)),
            ],
        )
        .await?;

    let rows: Vec<serde_json::Value> = res.take(1).unwrap_or_default();
    let stored = rows
        .first()
        .and_then(|v| v.get("body_text"))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    if stored == plaintext {
        return Err("ciphertext_storage_failed".into());
    }

    let decrypted2 = CryptoEngine::decrypt_field_with_key(stored, &key)?;
    if decrypted2 != plaintext {
        return Err("ciphertext_decrypt_mismatch".into());
    }

    Ok(())
}

async fn libreofficekit_mock_execution(
    db: &neqst_core::orchestrator::Db,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tokio::fs::create_dir_all("data/simulator").await?;
    let doc_path = "data/simulator/demo.ods";
    tokio::fs::write(doc_path, b"neqst-demo-ods").await?;

    let meta = lok::parse_and_store_document_meta(db, doc_path).await?;
    if meta.storage_path != doc_path {
        return Err("document_meta_storage_path_mismatch".into());
    }

    let mut res = db
        .query("CREATE mail CONTENT { message_id: 'sim-from', subject: 'from', body_text: '', body_html: '', date_received: time::now(), is_read: false, size: 1, encryption_metadata: { e2ee: false } };")
        .await?;
    let from_rows: Vec<serde_json::Value> = res.take(0).unwrap_or_default();
    let from_id = from_rows
        .first()
        .and_then(|v| v.get("id"))
        .ok_or("sim_from_id_missing")?
        .to_string()
        .trim_matches('"')
        .to_string();

    let Some(to_id) = meta.record_id.as_ref().map(|s| s.trim_matches('"').to_string()) else {
        return Err("doc_id_missing".into());
    };

    let sql = r#"
RELATE $from -> linked_to -> $to SET created_at = time::now(), context_anchor = $anchor;
SELECT * FROM linked_to WHERE in = $from AND out = $to LIMIT 1;
"#;

    let from_thing: surrealdb::sql::Thing = from_id.parse()?;
    let to_thing: surrealdb::sql::Thing = to_id.parse()?;

    let mut rel_res = db
        .query_bind(
            sql,
            vec![
                ("from".to_string(), surrealdb::sql::Value::from(from_thing)),
                ("to".to_string(), surrealdb::sql::Value::from(to_thing)),
                (
                    "anchor".to_string(),
                    surrealdb::sql::Value::from("sheet:cell=R1C1"),
                ),
            ],
        )
        .await?;

    let rel_rows: Vec<serde_json::Value> = rel_res.take(1).unwrap_or_default();
    if rel_rows.is_empty() {
        return Err("linked_to_edge_missing".into());
    }

    Ok(())
}

async fn p2p_guard_check() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    std::env::remove_var("NEQST_REMOTE_TOKEN");
    std::env::remove_var("NEQST_MASTER_PUBKEY_ED25519_B64");

    let guard = RemoteClientGuard::from_env();
    let headers = axum::http::HeaderMap::new();

    let local_peer: SocketAddr = "127.0.0.1:50000".parse()?;
    let local_mode = guard.authorize_ws(local_peer, &headers, None).await?;
    if local_mode != neqst_core::network::ClientMode::Local {
        return Err("local_mode_expected".into());
    }

    let remote_peer: SocketAddr = "10.0.0.2:50000".parse()?;
    let remote_res = guard.authorize_ws(remote_peer, &headers, None).await;
    if remote_res.is_ok() {
        return Err("remote_should_reject_missing_token".into());
    }

    let forensic_path = std::path::Path::new("../ForensicData/neqst_forensics.ndjson");
    let content = tokio::fs::read_to_string(forensic_path).await.unwrap_or_default();
    if !content.contains("remote_auth_missing_token") {
        return Err("forensic_missing_remote_auth_event".into());
    }

    Ok(())
}

async fn worker_queue_check() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use tokio::sync::{mpsc, RwLock};

    std::env::set_var("NEQST_DB_ENGINE", "mem");
    let config = RuntimeConfig::from_env();
    let db0 = storage::init_db(&config).await?;
    storage::apply_schema(&db0).await?;
    let db = std::sync::Arc::new(db0);
    let crypto = std::sync::Arc::new(neqst_core::crypto::CryptoEngine::default());

    let sessions: queue::worker::SessionOutbox =
        std::sync::Arc::new(RwLock::new(std::collections::HashMap::new()));

    let (out_tx, mut out_rx) = mpsc::unbounded_channel::<String>();
    let session_id = queue::new_session_id();
    {
        let mut map = sessions.write().await;
        map.insert(session_id.clone(), out_tx);
    }

    let (task_tx, task_rx) = mpsc::channel(8);
    queue::worker::spawn_worker(db, crypto, sessions, task_rx);

    let task_id = queue::new_task_id();
    task_tx
        .send(queue::TaskEnvelope {
            session_id: session_id.clone(),
            task_id: task_id.clone(),
            task: queue::OfficeTask::NetworkSyncDelta {
                peer_id: "sim-peer".to_string(),
            },
        })
        .await?;

    let done = tokio::time::timeout(std::time::Duration::from_secs(3), async move {
        while let Some(msg) = out_rx.recv().await {
            if msg.contains("\"Completed\"") && msg.contains(&task_id) {
                return true;
            }
        }
        false
    })
    .await
    .unwrap_or(false);

    if !done {
        return Err("worker_queue_no_completion".into());
    }

    Ok(())
}

async fn bootstrap_node_status_check() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    std::env::set_var("NEQST_DB_ENGINE", "mem");
    std::env::set_var("NEQST_RPC_BIND", "127.0.0.1:0");
    std::env::set_var("NEQST_USER_KEY_B64", "BwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwcHBwc");

    let fake_lo_dir = std::path::Path::new("data/simulator/fake_lo/program");
    tokio::fs::create_dir_all(fake_lo_dir).await?;
    let fake_lib = if cfg!(target_os = "windows") {
        fake_lo_dir.join("libreofficekit.dll")
    } else if cfg!(target_os = "macos") {
        fake_lo_dir.join("libreofficekit.dylib")
    } else {
        fake_lo_dir.join("libreofficekit.so")
    };
    tokio::fs::write(&fake_lib, b"fake-lok-runtime").await?;
    std::env::set_var(
        "NEQST_LOK_FALLBACK_INSTALL_PATH",
        fake_lo_dir.to_string_lossy().to_string(),
    );

    let runtime = orchestrator::bootstrap_system_node().await?;
    let status = rpc::build_node_status_result(&runtime.state).await;

    if status.get("db_status").and_then(|v| v.as_str()) != Some("connected") {
        return Err("sys_get_node_status_db_status_invalid".into());
    }
    if status.get("lok_version").and_then(|v| v.as_str()) != Some("26.2.1") {
        return Err("sys_get_node_status_lok_version_invalid".into());
    }
    if status.get("crypto_status").and_then(|v| v.as_str()) != Some("active") {
        return Err("sys_get_node_status_crypto_status_invalid".into());
    }

    drop(runtime.listener);
    Ok(())
}

