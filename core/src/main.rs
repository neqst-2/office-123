<<<<<<< HEAD
use std::net::SocketAddr;

use axum::{routing::get, Router};
use tokio::signal;
use tracing_subscriber::EnvFilter;
use neqst_core::{orchestrator, rpc};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .json()
        .init();

    let runtime = orchestrator::bootstrap_system_node().await?;

    let app = Router::new()
        .route("/ws", get(rpc::ws_upgrade))
        .with_state(runtime.state);

    let local_addr: SocketAddr = runtime.listener.local_addr()?;
    tracing::info!(rpc_bind = %local_addr, "starting core rpc server");

    axum::serve(runtime.listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let _ = signal::ctrl_c().await;
}

=======
use std::{net::SocketAddr, sync::Arc};

use axum::{routing::get, Router};
use tokio::signal;
use tracing_subscriber::EnvFilter;

mod orchestrator;
mod crypto;
mod lok;
mod rpc;
mod storage;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .json()
        .init();

    let config = orchestrator::RuntimeConfig::from_env();
    let db = Arc::new(storage::init_db(&config).await?);
    storage::apply_schema(&db).await?;

    let crypto = Arc::new(crypto::CryptoEngine::from_env().unwrap_or_default());

    let state = orchestrator::AppState { db, crypto };

    let app = Router::new()
        .route("/ws", get(rpc::ws_upgrade))
        .with_state(state);

    let addr: SocketAddr = config.rpc_bind_addr.parse()?;
    tracing::info!(rpc_bind = %addr, "starting core rpc server");

    let listener = tokio::net::TcpListener::bind(addr).await?;

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let _ = signal::ctrl_c().await;
}

>>>>>>> 0dc035f57a1c694c8225272cdbd0bfc9c9d60bb9
