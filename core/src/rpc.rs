<<<<<<< HEAD
use axum::{
    extract::{
        connect_info::ConnectInfo,
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query,
        State,
    },
    response::IntoResponse,
};
use futures::{sink::SinkExt, stream::StreamExt};
use std::{collections::HashMap, net::SocketAddr};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use rand_core::RngCore;
use tokio::sync::mpsc;
use std::sync::atomic::Ordering;

use crate::crypto::CryptoEngine;
use crate::orchestrator::{self, AppState, Db};
use crate::network::{sign_paseto_v4_public, unix_now, ClientMode};
use crate::queue::{self, OfficeTask, TaskEnvelope};


#[derive(Debug, Deserialize)]
struct RpcRequest {
    id: String,
    method: String,
    #[serde(default)]
    params: Value,
}

#[derive(Debug, Serialize)]
struct RpcError {
    code: i64,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

#[derive(Debug, Serialize)]
struct RpcResponse {
    id: String,
    result: Option<Value>,
    error: Option<RpcError>,
}

pub async fn ws_upgrade(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: axum::http::HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let token = params.get("token").map(String::as_str);
    let mode = match state
        .remote_guard
        .authorize_ws(peer, &headers, token)
        .await
    {
        Ok(mode) => mode,
        Err(_) => {
            return axum::http::StatusCode::UNAUTHORIZED.into_response();
        }
    };

    ws.on_upgrade(move |socket| handle_socket(socket, state, peer, mode))
}

async fn handle_socket(socket: WebSocket, state: AppState, peer: SocketAddr, mode: ClientMode) {
    let session_id = queue::new_session_id();

    let (mut ws_tx, mut ws_rx) = socket.split();
    let (out_tx, mut out_rx) = mpsc::unbounded_channel::<String>();

    {
        let mut map = state.ws_sessions.write().await;
        map.insert(session_id.clone(), out_tx.clone());
    }

    let writer = tokio::spawn(async move {
        while let Some(text) = out_rx.recv().await {
            if ws_tx.send(Message::Text(text)).await.is_err() {
                break;
            }
        }
    });

    if mode == ClientMode::Remote {
        let mut peers = state.peers.write().await;
        peers.push(crate::network::PeerInfo {
            addr: peer.to_string(),
            mode,
            connected_at_unix: unix_now(),
        });
    }

    while let Some(msg) = ws_rx.next().await {
        let Ok(msg) = msg else { break };

        let Message::Text(text) = msg else {
            continue;
        };

        let response = match serde_json::from_str::<RpcRequest>(&text) {
            Ok(req) => dispatch(&state, mode, &session_id, req).await,
            Err(err) => RpcResponse {
                id: "invalid".to_string(),
                result: None,
                error: Some(RpcError {
                    code: -32700,
                    message: "parse_error".to_string(),
                    data: Some(json!({"detail": err.to_string()})),
                }),
            },
        };

        let Ok(out) = serde_json::to_string(&response) else { continue };
        if out_tx.send(out).is_err() {
            break;
        }
    }

    if mode == ClientMode::Remote {
        let mut peers = state.peers.write().await;
        peers.retain(|p| p.addr != peer.to_string());
    }

    {
        let mut map = state.ws_sessions.write().await;
        map.remove(&session_id);
    }

    let _ = writer.abort();
}

async fn dispatch(state: &AppState, mode: ClientMode, session_id: &str, req: RpcRequest) -> RpcResponse {
    match req.method.as_str() {
        "pim:get_unread_mails" => handle_get_unread_mails(&state.db, &state.crypto, req).await,
        "pim:get_agenda" => handle_get_agenda(&state.db, req).await,
        "pim:ingest_mail" => handle_ingest_mail(&state.db, &state.crypto, req).await,
        "doc:open_document" => handle_open_document(state, session_id, req).await,
        "sys:get_node_status" => handle_get_node_status(state, req).await,
        "sync:generate_remote_token" => handle_generate_remote_token(state, mode, req).await,
        "sync:list_peers" => handle_list_peers(state, req).await,
        "sync:push_changes" => handle_sync_push_changes(&state.db, req).await,
        "sync:pull_delta" => handle_sync_pull_delta(&state.db, req).await,
        _ => RpcResponse {
            id: req.id,
            result: None,
            error: Some(RpcError {
                code: -32601,
                message: "method_not_found".to_string(),
                data: Some(json!({"method": req.method})),
            }),
        },
    }
}

async fn handle_get_unread_mails(db: &Db, crypto: &CryptoEngine, req: RpcRequest) -> RpcResponse {
    let limit = req
        .params
        .get("limit")
        .and_then(Value::as_u64)
        .unwrap_or(50)
        .min(500);

    let sql = "SELECT id, message_id, subject, body_text, body_html, encryption_metadata, date_received, is_read, size FROM mail WHERE is_read = false ORDER BY date_received DESC LIMIT $limit";

    match db
        .query_bind(sql, vec![("limit".to_string(), surrealdb::sql::Value::from(limit))])
        .await
    {
        Ok(mut res) => {
            let mut items: Vec<Value> = res.take(0).unwrap_or_default();
            decrypt_mail_items(&mut items, crypto);
            RpcResponse {
                id: req.id,
                result: Some(json!({ "items": items })),
                error: None,
            }
        }
        Err(err) => RpcResponse {
            id: req.id,
            result: None,
            error: Some(RpcError {
                code: -32000,
                message: "db_error".to_string(),
                data: Some(json!({ "detail": err.to_string() })),
            }),
        },
    }
}

async fn handle_get_agenda(db: &Db, req: RpcRequest) -> RpcResponse {
    let Some(start) = req.params.get("start").and_then(Value::as_str) else {
        return RpcResponse {
            id: req.id,
            result: None,
            error: Some(RpcError {
                code: -32602,
                message: "invalid_params".to_string(),
                data: Some(json!({"missing": "start"})),
            }),
        };
    };

    let Some(end) = req.params.get("end").and_then(Value::as_str) else {
        return RpcResponse {
            id: req.id,
            result: None,
            error: Some(RpcError {
                code: -32602,
                message: "invalid_params".to_string(),
                data: Some(json!({"missing": "end"})),
            }),
        };
    };

    let sql = "SELECT id, title, description, start_time, end_time, location FROM calendar_event WHERE start_time >= $start AND start_time <= $end ORDER BY start_time ASC LIMIT 500";

    match db
        .query_bind(
            sql,
            vec![
                ("start".to_string(), surrealdb::sql::Value::from(start)),
                ("end".to_string(), surrealdb::sql::Value::from(end)),
            ],
        )
        .await
    {
        Ok(mut res) => {
            let items: Vec<Value> = res.take(0).unwrap_or_default();
            RpcResponse {
                id: req.id,
                result: Some(json!({ "items": items })),
                error: None,
            }
        }
        Err(err) => RpcResponse {
            id: req.id,
            result: None,
            error: Some(RpcError {
                code: -32000,
                message: "db_error".to_string(),
                data: Some(json!({ "detail": err.to_string() })),
            }),
        },
    }
}

async fn handle_ingest_mail(db: &Db, crypto: &CryptoEngine, req: RpcRequest) -> RpcResponse {
    let Some(message_id) = req.params.get("message_id").and_then(Value::as_str) else {
        return RpcResponse {
            id: req.id,
            result: None,
            error: Some(RpcError {
                code: -32602,
                message: "invalid_params".to_string(),
                data: Some(json!({"missing": "message_id"})),
            }),
        };
    };

    let subject = req.params.get("subject").and_then(Value::as_str).unwrap_or("");
    let body_text = req.params.get("body_text").and_then(Value::as_str).unwrap_or("");
    let body_html = req.params.get("body_html").and_then(Value::as_str).unwrap_or("");
    let date_received = req.params.get("date_received").and_then(Value::as_str).unwrap_or("");
    let size = req.params.get("size").and_then(Value::as_i64).unwrap_or(0);
    let is_read = req.params.get("is_read").and_then(Value::as_bool).unwrap_or(false);

    let (body_text_enc, body_html_enc, encryption_metadata) = if crypto.has_user_key() {
        let bt = crypto.encrypt_field(body_text);
        let bh = crypto.encrypt_field(body_html);
        match (bt, bh) {
            (Ok(bt), Ok(bh)) => (
                bt,
                bh,
                json!({"e2ee": true, "alg": "chacha20poly1305", "encoding": "b64", "v": 1}),
            ),
            _ => (
                body_text.to_string(),
                body_html.to_string(),
                json!({"e2ee": false, "error": "encrypt_failed"}),
            ),
        }
    } else {
        (
            body_text.to_string(),
            body_html.to_string(),
            json!({"e2ee": false, "error": "key_unavailable"}),
        )
    };

    let sql = r#"
UPSERT mail SET
  message_id = $message_id,
  subject = $subject,
  body_text = $body_text,
  body_html = $body_html,
  date_received = $date_received,
  is_read = $is_read,
  size = $size,
  encryption_metadata = $encryption_metadata
WHERE message_id = $message_id;
SELECT id, message_id, subject, date_received, is_read, size, encryption_metadata FROM mail WHERE message_id = $message_id LIMIT 1;
"#;

    match db
        .query_bind(
            sql,
            vec![
                ("message_id".to_string(), surrealdb::sql::Value::from(message_id)),
                ("subject".to_string(), surrealdb::sql::Value::from(subject)),
                ("body_text".to_string(), surrealdb::sql::Value::from(body_text_enc)),
                ("body_html".to_string(), surrealdb::sql::Value::from(body_html_enc)),
                ("date_received".to_string(), surrealdb::sql::Value::from(date_received)),
                ("is_read".to_string(), surrealdb::sql::Value::from(is_read)),
                ("size".to_string(), surrealdb::sql::Value::from(size)),
                ("encryption_metadata".to_string(), surrealdb::sql::Value::from(encryption_metadata)),
            ],
        )
        .await
    {
        Ok(mut res) => {
            let created: Vec<Value> = res.take(1).unwrap_or_default();
            RpcResponse {
                id: req.id,
                result: Some(json!({ "created": created })),
                error: None,
            }
        }
        Err(err) => RpcResponse {
            id: req.id,
            result: None,
            error: Some(RpcError {
                code: -32000,
                message: "db_error".to_string(),
                data: Some(json!({ "detail": err.to_string() })),
            }),
        },
    }
}

async fn handle_open_document(state: &AppState, session_id: &str, req: RpcRequest) -> RpcResponse {
    let Some(path) = req.params.get("path").and_then(Value::as_str) else {
        return RpcResponse {
            id: req.id,
            result: None,
            error: Some(RpcError {
                code: -32602,
                message: "invalid_params".to_string(),
                data: Some(json!({"missing": "path"})),
            }),
        };
    };

    let link_ctx = req.params.get("link").map(|v| v.to_string());
    let task_id = queue::new_task_id();

    let env = TaskEnvelope {
        session_id: session_id.to_string(),
        task_id: task_id.clone(),
        task: OfficeTask::ProcessDocument {
            path: path.to_string(),
            link_ctx,
        },
    };

    state.queue_backlog.fetch_add(1, Ordering::Relaxed);
    if state.task_tx.send(env).await.is_err() {
        state.queue_backlog.fetch_sub(1, Ordering::Relaxed);
        return RpcResponse {
            id: req.id,
            result: None,
            error: Some(RpcError {
                code: -32000,
                message: "queue_unavailable".to_string(),
                data: None,
            }),
        };
    }

    RpcResponse {
        id: req.id,
        result: Some(json!({ "status": "queued", "task_id": task_id })),
        error: None,
    }
}

async fn handle_get_node_status(state: &AppState, req: RpcRequest) -> RpcResponse {
    RpcResponse {
        id: req.id,
        result: Some(build_node_status_result(state).await),
        error: None,
    }
}

pub async fn build_node_status_result(state: &AppState) -> Value {
    let health = orchestrator::get_node_health(state).await;
    json!({
        "db_status": health.db_status,
        "lok_status": health.lok_status,
        "lok_version": health.lok_version,
        "crypto_status": health.crypto_status,
        "queue_backlog": health.queue_backlog,
        "active_peers": health.active_peers
    })
}

fn decrypt_mail_items(items: &mut Vec<Value>, crypto: &CryptoEngine) {
    for item in items.iter_mut() {
        let Some(obj) = item.as_object_mut() else { continue };

        let is_e2ee = obj
            .get("encryption_metadata")
            .and_then(|v| v.get("e2ee"))
            .and_then(Value::as_bool)
            .unwrap_or(false);

        let raw_text = obj.get("body_text").and_then(Value::as_str).map(str::to_string);
        let raw_html = obj.get("body_html").and_then(Value::as_str).map(str::to_string);

        if is_e2ee && crypto.has_user_key() {
            if let Some(raw) = raw_text.as_deref() {
                if let Ok(plain) = crypto.decrypt_field(raw) {
                    obj.insert("body_text_plain".to_string(), Value::String(plain));
                }
                obj.insert("body_text_raw".to_string(), Value::String(raw.to_string()));
            }
            if let Some(raw) = raw_html.as_deref() {
                if let Ok(plain) = crypto.decrypt_field(raw) {
                    obj.insert("body_html_plain".to_string(), Value::String(plain));
                }
                obj.insert("body_html_raw".to_string(), Value::String(raw.to_string()));
            }
        } else {
            if let Some(raw) = raw_text {
                obj.insert("body_text_raw".to_string(), Value::String(raw));
            }
            if let Some(raw) = raw_html {
                obj.insert("body_html_raw".to_string(), Value::String(raw));
            }
        }

        obj.insert("e2ee_protected".to_string(), Value::Bool(is_e2ee));
    }
}

async fn handle_generate_remote_token(state: &AppState, mode: ClientMode, req: RpcRequest) -> RpcResponse {
    if mode != ClientMode::Local {
        return RpcResponse {
            id: req.id,
            result: None,
            error: Some(RpcError {
                code: -32601,
                message: "method_not_available_remote".to_string(),
                data: None,
            }),
        };
    }

    let ttl_seconds = req
        .params
        .get("ttl_seconds")
        .and_then(Value::as_u64)
        .unwrap_or(900)
        .min(1800);

    let now = unix_now();
    let exp = now + ttl_seconds;

    let mut nonce_bytes = [0u8; 16];
    rand_core::OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(nonce_bytes);

    let payload = json!({
        "exp": exp,
        "iat": now,
        "aud": "neqst-remote",
        "nonce": nonce,
    });

    let Some(signing) = load_master_signing_key() else {
        return RpcResponse {
            id: req.id,
            result: None,
            error: Some(RpcError {
                code: -32000,
                message: "signing_key_unavailable".to_string(),
                data: Some(json!({"expected_env": "NEQST_MASTER_PRIVKEY_ED25519_B64"})),
            }),
        };
    };

    let payload_bytes = payload.to_string().into_bytes();
    let token = sign_paseto_v4_public(&payload_bytes, &signing);

    let _ = state
        .remote_guard
        .forensic_flag("remote_token_generated_local", "127.0.0.1:0".parse().unwrap(), json!({"exp": exp}))
        .await;

    RpcResponse {
        id: req.id,
        result: Some(json!({ "token": token, "expires_unix": exp })),
        error: None,
    }
}

async fn handle_list_peers(state: &AppState, req: RpcRequest) -> RpcResponse {
    let peers = state.peers.read().await;
    RpcResponse {
        id: req.id,
        result: Some(json!({ "peers": &*peers })),
        error: None,
    }
}

async fn handle_sync_push_changes(db: &Db, req: RpcRequest) -> RpcResponse {
    let docs = req.params.get("documents").and_then(Value::as_array).cloned().unwrap_or_default();
    let edges = req.params.get("edges").and_then(Value::as_array).cloned().unwrap_or_default();

    let mut sql = String::from("BEGIN TRANSACTION;\n");
    let mut bindings: Vec<(String, surrealdb::sql::Value)> = Vec::new();

    for (i, doc) in docs.iter().enumerate() {
        let Some(obj) = doc.as_object() else { continue };
        let sp = obj.get("storage_path").and_then(Value::as_str).unwrap_or("");
        let fnm = obj.get("filename").and_then(Value::as_str).unwrap_or("");
        let mime = obj.get("mime_type").and_then(Value::as_str).unwrap_or("application/octet-stream");
        let size = obj.get("file_size").and_then(Value::as_i64).unwrap_or(0);

        let k_sp = format!("d{}_sp", i);
        let k_fn = format!("d{}_fn", i);
        let k_mi = format!("d{}_mi", i);
        let k_sz = format!("d{}_sz", i);

        sql.push_str(&format!(
            "UPSERT document_meta SET filename = ${}, file_size = ${}, storage_path = ${}, mime_type = ${}, last_modified = time::now() WHERE storage_path = ${};\n",
            k_fn, k_sz, k_sp, k_mi, k_sp
        ));

        bindings.push((k_sp.clone(), surrealdb::sql::Value::from(sp)));
        bindings.push((k_fn.clone(), surrealdb::sql::Value::from(fnm)));
        bindings.push((k_mi.clone(), surrealdb::sql::Value::from(mime)));
        bindings.push((k_sz.clone(), surrealdb::sql::Value::from(size)));
    }

    for (i, edge) in edges.iter().enumerate() {
        let Some(obj) = edge.as_object() else { continue };
        let from = obj.get("from").and_then(Value::as_str).unwrap_or("");
        let to = obj.get("to").and_then(Value::as_str).unwrap_or("");
        let anchor = obj.get("context_anchor").and_then(Value::as_str).unwrap_or("");

        let Ok(from_thing) = from.parse::<surrealdb::sql::Thing>() else { continue };
        let Ok(to_thing) = to.parse::<surrealdb::sql::Thing>() else { continue };

        let k_from = format!("e{}_from", i);
        let k_to = format!("e{}_to", i);
        let k_an = format!("e{}_an", i);

        sql.push_str(&format!(
            "RELATE ${} -> linked_to -> ${} SET created_at = time::now(), context_anchor = ${};\n",
            k_from, k_to, k_an
        ));

        bindings.push((k_from.clone(), surrealdb::sql::Value::from(from_thing)));
        bindings.push((k_to.clone(), surrealdb::sql::Value::from(to_thing)));
        bindings.push((k_an.clone(), surrealdb::sql::Value::from(anchor)));
    }

    sql.push_str("COMMIT TRANSACTION;\n");
    sql.push_str("SELECT { documents: $documents_count, edges: $edges_count } AS summary;\n");

    bindings.push((
        "documents_count".to_string(),
        surrealdb::sql::Value::from(docs.len() as i64),
    ));
    bindings.push(("edges_count".to_string(), surrealdb::sql::Value::from(edges.len() as i64)));

    match db.query_bind(&sql, bindings).await {
        Ok(mut res) => {
            let out: Vec<Value> = res.take(0).unwrap_or_default();
            RpcResponse {
                id: req.id,
                result: Some(json!({ "applied": out })),
                error: None,
            }
        }
        Err(err) => RpcResponse {
            id: req.id,
            result: None,
            error: Some(RpcError {
                code: -32000,
                message: "db_error".to_string(),
                data: Some(json!({ "detail": err.to_string() })),
            }),
        },
    }
}

async fn handle_sync_pull_delta(db: &Db, req: RpcRequest) -> RpcResponse {
    let since = req.params.get("since").and_then(Value::as_str).unwrap_or("1970-01-01T00:00:00Z");

    let sql = r#"
SELECT * FROM document_meta WHERE last_modified > time::parse($since) ORDER BY last_modified ASC LIMIT 1000;
SELECT in, out, context_anchor, created_at FROM linked_to WHERE created_at > time::parse($since) ORDER BY created_at ASC LIMIT 2000;
"#;

    match db
        .query_bind(sql, vec![("since".to_string(), surrealdb::sql::Value::from(since))])
        .await
    {
        Ok(mut res) => {
            let docs: Vec<Value> = res.take(0).unwrap_or_default();
            let edges: Vec<Value> = res.take(1).unwrap_or_default();
            RpcResponse {
                id: req.id,
                result: Some(json!({ "documents": docs, "edges": edges })),
                error: None,
            }
        }
        Err(err) => RpcResponse {
            id: req.id,
            result: None,
            error: Some(RpcError {
                code: -32000,
                message: "db_error".to_string(),
                data: Some(json!({ "detail": err.to_string() })),
            }),
        },
    }
}

fn load_master_signing_key() -> Option<ed25519_dalek::SigningKey> {
    let key_b64 = std::env::var("NEQST_MASTER_PRIVKEY_ED25519_B64").ok()?;
    if key_b64.trim().is_empty() {
        return None;
    }
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(key_b64.trim())
        .or_else(|_| base64::engine::general_purpose::STANDARD_NO_PAD.decode(key_b64.trim()))
        .ok()?;
    let sk: [u8; 32] = bytes.as_slice().try_into().ok()?;
    Some(ed25519_dalek::SigningKey::from_bytes(&sk))
}

=======
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::crypto::CryptoEngine;
use crate::orchestrator::{AppState, Db};
use crate::lok;

#[derive(Debug, Deserialize)]
struct RpcRequest {
    id: String,
    method: String,
    #[serde(default)]
    params: Value,
}

#[derive(Debug, Serialize)]
struct RpcError {
    code: i64,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

#[derive(Debug, Serialize)]
struct RpcResponse {
    id: String,
    result: Option<Value>,
    error: Option<RpcError>,
}

pub async fn ws_upgrade(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    while let Some(msg) = socket.recv().await {
        let Ok(msg) = msg else { break };

        let Message::Text(text) = msg else {
            continue;
        };

        let response = match serde_json::from_str::<RpcRequest>(&text) {
            Ok(req) => dispatch(&state, req).await,
            Err(err) => RpcResponse {
                id: "invalid".to_string(),
                result: None,
                error: Some(RpcError {
                    code: -32700,
                    message: "parse_error".to_string(),
                    data: Some(json!({"detail": err.to_string()})),
                }),
            },
        };

        let Ok(out) = serde_json::to_string(&response) else { continue };
        let _ = socket.send(Message::Text(out)).await;
    }
}

async fn dispatch(state: &AppState, req: RpcRequest) -> RpcResponse {
    match req.method.as_str() {
        "pim:get_unread_mails" => handle_get_unread_mails(&state.db, &state.crypto, req).await,
        "pim:get_agenda" => handle_get_agenda(&state.db, req).await,
        "pim:ingest_mail" => handle_ingest_mail(&state.db, &state.crypto, req).await,
        "doc:open_document" => handle_open_document(&state.db, req).await,
        _ => RpcResponse {
            id: req.id,
            result: None,
            error: Some(RpcError {
                code: -32601,
                message: "method_not_found".to_string(),
                data: Some(json!({"method": req.method})),
            }),
        },
    }
}

async fn handle_get_unread_mails(db: &Db, crypto: &CryptoEngine, req: RpcRequest) -> RpcResponse {
    let limit = req
        .params
        .get("limit")
        .and_then(Value::as_u64)
        .unwrap_or(50)
        .min(500);

    let sql = "SELECT id, message_id, subject, body_text, body_html, encryption_metadata, date_received, is_read, size FROM mail WHERE is_read = false ORDER BY date_received DESC LIMIT $limit";

    match db
        .query_bind(sql, vec![("limit", surrealdb::sql::Value::from(limit))])
        .await
    {
        Ok(mut res) => {
            let mut items: Vec<Value> = res.take(0).unwrap_or_default();
            decrypt_mail_items(&mut items, crypto);
            RpcResponse {
                id: req.id,
                result: Some(json!({ "items": items })),
                error: None,
            }
        }
        Err(err) => RpcResponse {
            id: req.id,
            result: None,
            error: Some(RpcError {
                code: -32000,
                message: "db_error".to_string(),
                data: Some(json!({ "detail": err.to_string() })),
            }),
        },
    }
}

async fn handle_get_agenda(db: &Db, req: RpcRequest) -> RpcResponse {
    let Some(start) = req.params.get("start").and_then(Value::as_str) else {
        return RpcResponse {
            id: req.id,
            result: None,
            error: Some(RpcError {
                code: -32602,
                message: "invalid_params".to_string(),
                data: Some(json!({"missing": "start"})),
            }),
        };
    };

    let Some(end) = req.params.get("end").and_then(Value::as_str) else {
        return RpcResponse {
            id: req.id,
            result: None,
            error: Some(RpcError {
                code: -32602,
                message: "invalid_params".to_string(),
                data: Some(json!({"missing": "end"})),
            }),
        };
    };

    let sql = "SELECT id, title, description, start_time, end_time, location FROM calendar_event WHERE start_time >= $start AND start_time <= $end ORDER BY start_time ASC LIMIT 500";

    match db
        .query_bind(
            sql,
            vec![
                ("start", surrealdb::sql::Value::from(start)),
                ("end", surrealdb::sql::Value::from(end)),
            ],
        )
        .await
    {
        Ok(mut res) => {
            let items: Vec<Value> = res.take(0).unwrap_or_default();
            RpcResponse {
                id: req.id,
                result: Some(json!({ "items": items })),
                error: None,
            }
        }
        Err(err) => RpcResponse {
            id: req.id,
            result: None,
            error: Some(RpcError {
                code: -32000,
                message: "db_error".to_string(),
                data: Some(json!({ "detail": err.to_string() })),
            }),
        },
    }
}

async fn handle_ingest_mail(db: &Db, crypto: &CryptoEngine, req: RpcRequest) -> RpcResponse {
    let Some(message_id) = req.params.get("message_id").and_then(Value::as_str) else {
        return RpcResponse {
            id: req.id,
            result: None,
            error: Some(RpcError {
                code: -32602,
                message: "invalid_params".to_string(),
                data: Some(json!({"missing": "message_id"})),
            }),
        };
    };

    let subject = req.params.get("subject").and_then(Value::as_str).unwrap_or("");
    let body_text = req.params.get("body_text").and_then(Value::as_str).unwrap_or("");
    let body_html = req.params.get("body_html").and_then(Value::as_str).unwrap_or("");
    let date_received = req.params.get("date_received").and_then(Value::as_str).unwrap_or("");
    let size = req.params.get("size").and_then(Value::as_i64).unwrap_or(0);
    let is_read = req.params.get("is_read").and_then(Value::as_bool).unwrap_or(false);

    let (body_text_enc, body_html_enc, encryption_metadata) = if crypto.has_user_key() {
        let bt = crypto.encrypt_field(body_text);
        let bh = crypto.encrypt_field(body_html);
        match (bt, bh) {
            (Ok(bt), Ok(bh)) => (
                bt,
                bh,
                json!({"e2ee": true, "alg": "chacha20poly1305", "encoding": "b64", "v": 1}),
            ),
            _ => (
                body_text.to_string(),
                body_html.to_string(),
                json!({"e2ee": false, "error": "encrypt_failed"}),
            ),
        }
    } else {
        (
            body_text.to_string(),
            body_html.to_string(),
            json!({"e2ee": false, "error": "key_unavailable"}),
        )
    };

    let sql = r#"
UPSERT mail SET
  message_id = $message_id,
  subject = $subject,
  body_text = $body_text,
  body_html = $body_html,
  date_received = $date_received,
  is_read = $is_read,
  size = $size,
  encryption_metadata = $encryption_metadata
WHERE message_id = $message_id;
SELECT id, message_id, subject, date_received, is_read, size, encryption_metadata FROM mail WHERE message_id = $message_id LIMIT 1;
"#;

    match db
        .query_bind(
            sql,
            vec![
                ("message_id", surrealdb::sql::Value::from(message_id)),
                ("subject", surrealdb::sql::Value::from(subject)),
                ("body_text", surrealdb::sql::Value::from(body_text_enc)),
                ("body_html", surrealdb::sql::Value::from(body_html_enc)),
                ("date_received", surrealdb::sql::Value::from(date_received)),
                ("is_read", surrealdb::sql::Value::from(is_read)),
                ("size", surrealdb::sql::Value::from(size)),
                ("encryption_metadata", surrealdb::sql::Value::from(encryption_metadata)),
            ],
        )
        .await
    {
        Ok(mut res) => {
            let created: Vec<Value> = res.take(1).unwrap_or_default();
            RpcResponse {
                id: req.id,
                result: Some(json!({ "created": created })),
                error: None,
            }
        }
        Err(err) => RpcResponse {
            id: req.id,
            result: None,
            error: Some(RpcError {
                code: -32000,
                message: "db_error".to_string(),
                data: Some(json!({ "detail": err.to_string() })),
            }),
        },
    }
}

async fn handle_open_document(db: &Db, req: RpcRequest) -> RpcResponse {
    let Some(path) = req.params.get("path").and_then(Value::as_str) else {
        return RpcResponse {
            id: req.id,
            result: None,
            error: Some(RpcError {
                code: -32602,
                message: "invalid_params".to_string(),
                data: Some(json!({"missing": "path"})),
            }),
        };
    };

    let filename = req
        .params
        .get("filename")
        .and_then(Value::as_str)
        .map(str::to_string)
        .unwrap_or_else(|| PathLike::basename(path));

    let extract = match lok::parse_and_store_document_meta(db, path).await {
        Ok(v) => v,
        Err(err) => {
            return RpcResponse {
                id: req.id,
                result: None,
                error: Some(RpcError {
                    code: -32000,
                    message: "lok_or_db_error".to_string(),
                    data: Some(json!({ "detail": err.to_string() })),
                }),
            };
        }
    };

    let mut linked = false;
    if let (Some(link), Some(doc_id_str)) = (req.params.get("link"), extract.record_id.as_ref()) {
        if let (Some(from_str), Some(anchor)) = (
            link.get("from").and_then(Value::as_str),
            link.get("context_anchor").and_then(Value::as_str),
        ) {
            if let (Ok(from), Ok(to)) = (
                from_str.parse::<surrealdb::sql::Thing>(),
                doc_id_str.trim_matches('"').parse::<surrealdb::sql::Thing>(),
            ) {
                let sql = r#"
RELATE $from -> linked_to -> $to SET created_at = time::now(), context_anchor = $anchor;
"#;
                let _ = db
                    .query_bind(
                        sql,
                        vec![
                            ("from", surrealdb::sql::Value::from(from)),
                            ("to", surrealdb::sql::Value::from(to)),
                            ("anchor", surrealdb::sql::Value::from(anchor)),
                        ],
                    )
                    .await;
                linked = true;
            }
        }
    }

    RpcResponse {
        id: req.id,
        result: Some(json!({
            "meta": {
                "id": extract.record_id,
                "filename": filename,
                "storage_path": extract.storage_path,
                "mime_type": extract.mime_type,
                "file_size": extract.file_size,
            },
            "lok": {
                "available": extract.lok_available,
                "parts": extract.parts,
                "structure_hash": extract.structure_hash,
            },
            "is_graph_enhanced": extract.is_graph_enhanced,
            "linked_to_inserted": linked
        })),
        error: None,
    }
}

struct PathLike;

impl PathLike {
    fn basename(path: &str) -> String {
        path.rsplit(['/', '\\']).next().unwrap_or(path).to_string()
    }
}

fn decrypt_mail_items(items: &mut Vec<Value>, crypto: &CryptoEngine) {
    for item in items.iter_mut() {
        let Some(obj) = item.as_object_mut() else { continue };

        let is_e2ee = obj
            .get("encryption_metadata")
            .and_then(|v| v.get("e2ee"))
            .and_then(Value::as_bool)
            .unwrap_or(false);

        let raw_text = obj.get("body_text").and_then(Value::as_str).map(str::to_string);
        let raw_html = obj.get("body_html").and_then(Value::as_str).map(str::to_string);

        if is_e2ee && crypto.has_user_key() {
            if let Some(raw) = raw_text.as_deref() {
                if let Ok(plain) = crypto.decrypt_field(raw) {
                    obj.insert("body_text_plain".to_string(), Value::String(plain));
                }
                obj.insert("body_text_raw".to_string(), Value::String(raw.to_string()));
            }
            if let Some(raw) = raw_html.as_deref() {
                if let Ok(plain) = crypto.decrypt_field(raw) {
                    obj.insert("body_html_plain".to_string(), Value::String(plain));
                }
                obj.insert("body_html_raw".to_string(), Value::String(raw.to_string()));
            }
        } else {
            if let Some(raw) = raw_text {
                obj.insert("body_text_raw".to_string(), Value::String(raw));
            }
            if let Some(raw) = raw_html {
                obj.insert("body_html_raw".to_string(), Value::String(raw));
            }
        }

        obj.insert("e2ee_protected".to_string(), Value::Bool(is_e2ee));
    }
}

>>>>>>> 0dc035f57a1c694c8225272cdbd0bfc9c9d60bb9
