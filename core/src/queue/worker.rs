use std::{
    collections::HashMap,
    sync::{atomic::{AtomicUsize, Ordering}, Arc},
};

use serde_json::{json, Value};
use tokio::sync::{mpsc, RwLock};

use crate::{
    crypto::CryptoEngine,
    lok,
    orchestrator::Db,
    queue::{OfficeTask, TaskEnvelope, TaskProgress},
};

pub type SessionOutbox = Arc<RwLock<HashMap<String, mpsc::UnboundedSender<String>>>>;

pub fn spawn_worker(
    db: Arc<Db>,
    crypto: Arc<CryptoEngine>,
    sessions: SessionOutbox,
    queue_backlog: Arc<AtomicUsize>,
    mut rx: mpsc::Receiver<TaskEnvelope>,
) {
    tokio::spawn(async move {
        while let Some(env) = rx.recv().await {
            let _ = queue_backlog.fetch_update(
                Ordering::Relaxed,
                Ordering::Relaxed,
                |value| value.checked_sub(1),
            );
            let _ = process_one(&db, &crypto, &sessions, env).await;
        }
    });
}

async fn process_one(
    db: &Db,
    crypto: &CryptoEngine,
    sessions: &SessionOutbox,
    env: TaskEnvelope,
) -> Result<(), ()> {
    notify_progress(
        sessions,
        &env.session_id,
        TaskProgress::Started {
            task_id: env.task_id.clone(),
        },
        None,
    )
    .await;

    match env.task {
        OfficeTask::ProcessDocument { path, link_ctx } => {
            notify_progress(
                sessions,
                &env.session_id,
                TaskProgress::Processing {
                    task_id: env.task_id.clone(),
                    percentage: 10,
                    status_message: "📄 Scheduling Headless LibreOfficeKit parsing...".to_string(),
                },
                None,
            )
            .await;

            let meta = match lok::parse_and_store_document_meta(db, &path).await {
                Ok(v) => v,
                Err(err) => {
                    notify_progress(
                        sessions,
                        &env.session_id,
                        TaskProgress::Failed {
                            task_id: env.task_id.clone(),
                            error_message: err.to_string(),
                        },
                        None,
                    )
                    .await;
                    return Err(());
                }
            };

            notify_progress(
                sessions,
                &env.session_id,
                TaskProgress::Processing {
                    task_id: env.task_id.clone(),
                    percentage: 75,
                    status_message: "🔗 Building graph context anchors...".to_string(),
                },
                None,
            )
            .await;

            if let (Some(link_json), Some(to_id_str)) = (link_ctx.as_ref(), meta.record_id.as_ref()) {
                if let Ok(link_val) = serde_json::from_str::<Value>(link_json) {
                    let from_str = link_val.get("from").and_then(Value::as_str).unwrap_or("");
                    let anchor = link_val
                        .get("context_anchor")
                        .and_then(Value::as_str)
                        .unwrap_or("");
                    if let (Ok(from), Ok(to)) = (
                        from_str.parse::<surrealdb::sql::Thing>(),
                        to_id_str.trim_matches('"').parse::<surrealdb::sql::Thing>(),
                    ) {
                        let _ = db
                            .query_bind(
                                "RELATE $from -> linked_to -> $to SET created_at = time::now(), context_anchor = $anchor;",
                                vec![
                                    ("from".to_string(), surrealdb::sql::Value::from(from)),
                                    ("to".to_string(), surrealdb::sql::Value::from(to)),
                                    (
                                        "anchor".to_string(),
                                        surrealdb::sql::Value::from(anchor),
                                    ),
                                ],
                            )
                            .await;
                    }
                }
            }

            let result = json!({
                "meta": {
                    "id": meta.record_id,
                    "filename": meta.filename,
                    "storage_path": meta.storage_path,
                    "mime_type": meta.mime_type,
                    "file_size": meta.file_size,
                },
                "lok": {
                    "available": meta.lok_available,
                    "parts": meta.parts,
                    "structure_hash": meta.structure_hash,
                },
                "is_graph_enhanced": meta.is_graph_enhanced
            });

            notify_progress(
                sessions,
                &env.session_id,
                TaskProgress::Completed {
                    task_id: env.task_id.clone(),
                    result_summary: "document_processed".to_string(),
                },
                Some(result),
            )
            .await;
            Ok(())
        }
        OfficeTask::BulkEncryptMails { mail_ids } => {
            if !crypto.has_user_key() {
                notify_progress(
                    sessions,
                    &env.session_id,
                    TaskProgress::Failed {
                        task_id: env.task_id.clone(),
                        error_message: "crypto_key_unavailable".to_string(),
                    },
                    None,
                )
                .await;
                return Err(());
            }

            let total = mail_ids.len().max(1);
            for (i, id_str) in mail_ids.iter().enumerate() {
                let pct = (((i + 1) * 100) / total).min(100) as u8;
                notify_progress(
                    sessions,
                    &env.session_id,
                    TaskProgress::Processing {
                        task_id: env.task_id.clone(),
                        percentage: pct,
                        status_message: format!("🔒 Encrypting mail {}", id_str),
                    },
                    None,
                )
                .await;

                let Ok(thing) = id_str.parse::<surrealdb::sql::Thing>() else { continue };
                let sql = "SELECT body_text, body_html, encryption_metadata FROM $id LIMIT 1;";
                let mut res = match db
                    .query_bind(sql, vec![("id".to_string(), surrealdb::sql::Value::from(thing.clone()))])
                    .await
                {
                    Ok(v) => v,
                    Err(_) => continue,
                };

                let rows: Vec<Value> = res.take(0).unwrap_or_default();
                let Some(row) = rows.first().and_then(|v| v.as_object()) else { continue };
                let is_e2ee = row
                    .get("encryption_metadata")
                    .and_then(|v| v.get("e2ee"))
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                if is_e2ee {
                    continue;
                }
                let bt = row.get("body_text").and_then(Value::as_str).unwrap_or("");
                let bh = row.get("body_html").and_then(Value::as_str).unwrap_or("");

                let Ok(bt_enc) = crypto.encrypt_field(bt) else { continue };
                let Ok(bh_enc) = crypto.encrypt_field(bh) else { continue };

                let _ = db
                    .query_bind(
                        "UPDATE $id SET body_text = $bt, body_html = $bh, encryption_metadata = { e2ee: true, alg: 'chacha20poly1305', encoding: 'b64', v: 1 };",
                        vec![
                            ("id".to_string(), surrealdb::sql::Value::from(thing)),
                            ("bt".to_string(), surrealdb::sql::Value::from(bt_enc)),
                            ("bh".to_string(), surrealdb::sql::Value::from(bh_enc)),
                        ],
                    )
                    .await;
            }

            notify_progress(
                sessions,
                &env.session_id,
                TaskProgress::Completed {
                    task_id: env.task_id.clone(),
                    result_summary: "mails_encrypted".to_string(),
                },
                None,
            )
            .await;
            Ok(())
        }
        OfficeTask::NetworkSyncDelta { peer_id } => {
            notify_progress(
                sessions,
                &env.session_id,
                TaskProgress::Processing {
                    task_id: env.task_id.clone(),
                    percentage: 50,
                    status_message: format!("🌐 Syncing delta for peer {}", peer_id),
                },
                None,
            )
            .await;

            notify_progress(
                sessions,
                &env.session_id,
                TaskProgress::Completed {
                    task_id: env.task_id.clone(),
                    result_summary: "sync_delta_placeholder".to_string(),
                },
                None,
            )
            .await;
            Ok(())
        }
    }
}

async fn notify_progress(
    sessions: &SessionOutbox,
    session_id: &str,
    progress: TaskProgress,
    result: Option<Value>,
) {
    let msg = if let Some(result) = result {
        json!({
            "method": "task:progress",
            "params": { "progress": progress, "result": result }
        })
    } else {
        json!({
            "method": "task:progress",
            "params": { "progress": progress }
        })
    };

    let text = match serde_json::to_string(&msg) {
        Ok(v) => v,
        Err(_) => return,
    };

    let tx = {
        let map = sessions.read().await;
        map.get(session_id).cloned()
    };

    if let Some(tx) = tx {
        let _ = tx.send(text);
    }
}

