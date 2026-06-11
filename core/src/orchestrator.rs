<<<<<<< HEAD
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use surrealdb::engine::local::{Mem, RocksDb};
use surrealdb::{Response, Surreal};
use surrealdb::sql::Value;
use tokio::sync::mpsc;

use crate::crypto::CryptoEngine;
use crate::lok::lifecycle::CoreVersionManager;
use crate::network::{PeerInfo, RemoteClientGuard};
use crate::queue::{self, TaskEnvelope};

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Db>,
    pub crypto: Arc<CryptoEngine>,
    pub remote_guard: Arc<RemoteClientGuard>,
    pub peers: Arc<tokio::sync::RwLock<Vec<PeerInfo>>>,
    pub task_tx: mpsc::Sender<TaskEnvelope>,
    pub ws_sessions: crate::queue::worker::SessionOutbox,
    pub health_status: Arc<tokio::sync::RwLock<NodeHealthSnapshot>>,
    pub queue_backlog: Arc<AtomicUsize>,
}

#[derive(Clone)]
pub struct BootstrapRuntime {
    pub state: AppState,
    pub listener: tokio::net::TcpListener,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct NodeHealthSnapshot {
    pub db_status: String,
    pub lok_status: String,
    pub lok_version: String,
    pub crypto_status: String,
    pub queue_backlog: usize,
    pub active_peers: usize,
}

impl Default for NodeHealthSnapshot {
    fn default() -> Self {
        Self {
            db_status: "initializing".to_string(),
            lok_status: "initializing".to_string(),
            lok_version: "unknown".to_string(),
            crypto_status: "initializing".to_string(),
            queue_backlog: 0,
            active_peers: 0,
        }
    }
}

#[derive(Debug)]
pub enum BootError {
    Environment(String),
    Database(String),
    Queue(String),
    Network(String),
}

impl std::fmt::Display for BootError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BootError::Environment(v) => write!(f, "environment_boot_error: {}", v),
            BootError::Database(v) => write!(f, "database_boot_error: {}", v),
            BootError::Queue(v) => write!(f, "queue_boot_error: {}", v),
            BootError::Network(v) => write!(f, "network_boot_error: {}", v),
        }
    }
}

impl std::error::Error for BootError {}

pub enum Db {
    Rocks(Surreal<RocksDb>),
    Mem(Surreal<Mem>),
}

impl Db {
    pub async fn query(&self, sql: &str) -> Result<Response, surrealdb::Error> {
        match self {
            Db::Rocks(db) => db.query(sql).await,
            Db::Mem(db) => db.query(sql).await,
        }
    }

    pub async fn query_bind(
        &self,
        sql: &str,
        bindings: Vec<(String, Value)>,
    ) -> Result<Response, surrealdb::Error> {
        match self {
            Db::Rocks(db) => {
                let mut q = db.query(sql);
                for (k, v) in bindings {
                    q = q.bind((k.as_str(), v));
                }
                q.await
            }
            Db::Mem(db) => {
                let mut q = db.query(sql);
                for (k, v) in bindings {
                    q = q.bind((k.as_str(), v));
                }
                q.await
            }
        }
    }
}

#[derive(Clone)]
pub struct RuntimeConfig {
    pub rpc_bind_addr: String,
    pub db_engine: String,
    pub db_path: String,
    pub surreal_namespace: String,
    pub surreal_database: String,
}

impl RuntimeConfig {
    pub fn from_env() -> Self {
        let rpc_bind_addr = std::env::var("NEQST_RPC_BIND").unwrap_or_else(|_| "127.0.0.1:9001".to_string());
        let db_engine = std::env::var("NEQST_DB_ENGINE").unwrap_or_else(|_| "rocksdb".to_string());
        let db_path = std::env::var("NEQST_DB_PATH").unwrap_or_else(|_| "data/surrealdb".to_string());
        let surreal_namespace = std::env::var("NEQST_NS").unwrap_or_else(|_| "neqst".to_string());
        let surreal_database = std::env::var("NEQST_DB").unwrap_or_else(|_| "office".to_string());

        Self {
            rpc_bind_addr,
            db_engine,
            db_path,
            surreal_namespace,
            surreal_database,
        }
    }
}

pub async fn bootstrap_system_node() -> Result<BootstrapRuntime, BootError> {
    let config = RuntimeConfig::from_env();
    let mut health = NodeHealthSnapshot::default();

    // Phase A: Environment & Versioning
    let version_manager = CoreVersionManager::load()
        .await
        .map_err(|err| BootError::Environment(err.to_string()))?;
    let stable_install_path = version_manager.stable_install_path();
    version_manager.apply_install_path_env(&stable_install_path);
    health.lok_version = version_manager.current_stable_version().to_string();
    health.lok_status = match version_manager.verify_embedded_hash(&stable_install_path).await {
        Ok(()) => "ready".to_string(),
        Err(_) => {
            if stable_install_path.exists() {
                "ready".to_string()
            } else {
                "initializing".to_string()
            }
        }
    };

    // Phase B: Database & Security Engine
    let crypto = Arc::new(CryptoEngine::from_env().unwrap_or_default());
    health.crypto_status = if crypto.has_user_key() {
        "active".to_string()
    } else {
        "waiting_for_key".to_string()
    };

    let db = Arc::new(
        crate::storage::init_db(&config)
            .await
            .map_err(|err| BootError::Database(err.to_string()))?,
    );
    crate::storage::apply_schema(&db)
        .await
        .map_err(|err| BootError::Database(err.to_string()))?;
    health.db_status = "connected".to_string();

    // Phase C: Concurrency Engine
    let remote_guard = Arc::new(RemoteClientGuard::from_env());
    let peers = Arc::new(tokio::sync::RwLock::new(Vec::new()));
    let ws_sessions = Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new()));
    let queue_backlog = Arc::new(AtomicUsize::new(0));

    let (task_tx, task_rx) = mpsc::channel(128);
    queue::worker::spawn_worker(
        db.clone(),
        crypto.clone(),
        ws_sessions.clone(),
        queue_backlog.clone(),
        task_rx,
    );

    let health_status = Arc::new(tokio::sync::RwLock::new(NodeHealthSnapshot {
        queue_backlog: queue_backlog.load(Ordering::Relaxed),
        active_peers: 0,
        ..health.clone()
    }));

    // Phase D: Network Layer
    let listener = tokio::net::TcpListener::bind(&config.rpc_bind_addr)
        .await
        .map_err(|err| BootError::Network(err.to_string()))?;

    let state = AppState {
        db,
        crypto,
        remote_guard,
        peers,
        task_tx,
        ws_sessions,
        health_status: health_status.clone(),
        queue_backlog,
    };

    let health_json = serde_json::to_string(&health).unwrap_or_else(|_| "{}".to_string());
    tracing::info!(health = %health_json, "bootstrap_system_node_complete");

    Ok(BootstrapRuntime { state, listener })
}

pub async fn get_node_health(state: &AppState) -> NodeHealthSnapshot {
    let base = state.health_status.read().await.clone();
    NodeHealthSnapshot {
        queue_backlog: state.queue_backlog.load(Ordering::Relaxed),
        active_peers: state.peers.read().await.len(),
        ..base
    }
}

pub async fn connect(config: &RuntimeConfig) -> Result<Db, surrealdb::Error> {
    let engine = config.db_engine.to_ascii_lowercase();
    if engine == "mem" || engine == "memory" {
        let db = Surreal::new::<Mem>(()).await?;
        db.use_ns(&config.surreal_namespace)
            .use_db(&config.surreal_database)
            .await?;
        return Ok(Db::Mem(db));
    }

    let db = Surreal::new::<RocksDb>(&config.db_path).await?;
    db.use_ns(&config.surreal_namespace)
        .use_db(&config.surreal_database)
        .await?;
    Ok(Db::Rocks(db))
}

=======
use std::sync::Arc;

use surrealdb::engine::local::{Mem, RocksDb};
use surrealdb::{Response, Surreal};
use surrealdb::sql::Value;

use crate::crypto::CryptoEngine;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Db>,
    pub crypto: Arc<CryptoEngine>,
}

pub enum Db {
    Rocks(Surreal<RocksDb>),
    Mem(Surreal<Mem>),
}

impl Db {
    pub async fn query(&self, sql: &str) -> Result<Response, surrealdb::Error> {
        match self {
            Db::Rocks(db) => db.query(sql).await,
            Db::Mem(db) => db.query(sql).await,
        }
    }

    pub async fn query_bind(
        &self,
        sql: &str,
        bindings: Vec<(&str, Value)>,
    ) -> Result<Response, surrealdb::Error> {
        match self {
            Db::Rocks(db) => {
                let mut q = db.query(sql);
                for (k, v) in bindings {
                    q = q.bind((k, v));
                }
                q.await
            }
            Db::Mem(db) => {
                let mut q = db.query(sql);
                for (k, v) in bindings {
                    q = q.bind((k, v));
                }
                q.await
            }
        }
    }
}

#[derive(Clone)]
pub struct RuntimeConfig {
    pub rpc_bind_addr: String,
    pub db_engine: String,
    pub db_path: String,
    pub surreal_namespace: String,
    pub surreal_database: String,
}

impl RuntimeConfig {
    pub fn from_env() -> Self {
        let rpc_bind_addr = std::env::var("NEQST_RPC_BIND").unwrap_or_else(|_| "127.0.0.1:9001".to_string());
        let db_engine = std::env::var("NEQST_DB_ENGINE").unwrap_or_else(|_| "rocksdb".to_string());
        let db_path = std::env::var("NEQST_DB_PATH").unwrap_or_else(|_| "data/surrealdb".to_string());
        let surreal_namespace = std::env::var("NEQST_NS").unwrap_or_else(|_| "neqst".to_string());
        let surreal_database = std::env::var("NEQST_DB").unwrap_or_else(|_| "office".to_string());

        Self {
            rpc_bind_addr,
            db_engine,
            db_path,
            surreal_namespace,
            surreal_database,
        }
    }
}

pub async fn connect(config: &RuntimeConfig) -> Result<Db, surrealdb::Error> {
    let engine = config.db_engine.to_ascii_lowercase();
    if engine == "mem" || engine == "memory" {
        let db = Surreal::new::<Mem>(()).await?;
        db.use_ns(&config.surreal_namespace)
            .use_db(&config.surreal_database)
            .await?;
        return Ok(Db::Mem(db));
    }

    let db = Surreal::new::<RocksDb>(&config.db_path).await?;
    db.use_ns(&config.surreal_namespace)
        .use_db(&config.surreal_database)
        .await?;
    Ok(Db::Rocks(db))
}

>>>>>>> 0dc035f57a1c694c8225272cdbd0bfc9c9d60bb9
