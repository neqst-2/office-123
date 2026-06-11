<<<<<<< HEAD
use std::path::{Path, PathBuf};

use tokio::fs;

use crate::orchestrator::{self, Db, RuntimeConfig};

pub async fn init_db(config: &RuntimeConfig) -> Result<Db, surrealdb::Error> {
    orchestrator::connect(config).await
}

pub async fn apply_schema(db: &Db) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let schema_path = resolve_schema_path().ok_or("schema.surrealql not found")?;
    let schema = fs::read_to_string(schema_path).await?;
    let _ = db.query(&schema).await?;
    Ok(())
}

fn resolve_schema_path() -> Option<PathBuf> {
    let candidates = [
        PathBuf::from("schema.surrealql"),
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(Path::to_path_buf))
            .unwrap_or_default()
            .join("schema.surrealql"),
    ];

    for candidate in candidates {
        if candidate.is_file() {
            return Some(candidate);
        }
    }

    let mut dir = std::env::current_dir().ok()?;
    for _ in 0..8 {
        let candidate = dir.join("schema.surrealql");
        if candidate.is_file() {
            return Some(candidate);
        }
        if !dir.pop() {
            break;
        }
    }

    None
}

=======
use std::path::{Path, PathBuf};

use tokio::fs;

use crate::orchestrator::{self, Db, RuntimeConfig};

pub async fn init_db(config: &RuntimeConfig) -> Result<Db, surrealdb::Error> {
    orchestrator::connect(config).await
}

pub async fn apply_schema(db: &Db) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let schema_path = resolve_schema_path().ok_or("schema.surrealql not found")?;
    let schema = fs::read_to_string(schema_path).await?;
    let _ = db.query(&schema).await?;
    Ok(())
}

fn resolve_schema_path() -> Option<PathBuf> {
    let candidates = [
        PathBuf::from("schema.surrealql"),
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(Path::to_path_buf))
            .unwrap_or_default()
            .join("schema.surrealql"),
    ];

    for candidate in candidates {
        if candidate.is_file() {
            return Some(candidate);
        }
    }

    let mut dir = std::env::current_dir().ok()?;
    for _ in 0..8 {
        let candidate = dir.join("schema.surrealql");
        if candidate.is_file() {
            return Some(candidate);
        }
        if !dir.pop() {
            break;
        }
    }

    None
}

>>>>>>> 0dc035f57a1c694c8225272cdbd0bfc9c9d60bb9
