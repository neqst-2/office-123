use std::path::{Path, PathBuf};

use serde::Deserialize;
use tokio::fs;
use tokio::io::AsyncWriteExt;

#[derive(Clone, Debug, Deserialize)]
pub struct CompatibilityMatrix {
    pub current_stable_core: String,
    pub minimum_required_version: String,
    pub embedded_hash_blake3: String,
    pub allowed_upstream_releases: Vec<String>,
}

#[derive(Clone)]
pub struct CoreVersionManager {
    matrix: CompatibilityMatrix,
    project_root: PathBuf,
}

impl CoreVersionManager {
    pub async fn load() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let project_root = resolve_project_root().ok_or("project_root_not_found")?;
        let matrix_path = project_root
            .join("core")
            .join("compatibility_matrix.json");
        let raw = fs::read_to_string(&matrix_path)
            .await
            .or_else(|_| fs::read_to_string(project_root.join("config").join("compatibility_matrix.json")))?;
        let matrix: CompatibilityMatrix = serde_json::from_str(&raw)?;
        Ok(Self { matrix, project_root })
    }

    pub fn current_stable_version(&self) -> &str {
        &self.matrix.current_stable_core
    }

    pub fn candidate_install_path(&self) -> Option<PathBuf> {
        std::env::var("NEQST_LOK_CANDIDATE_INSTALL_PATH")
            .ok()
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
    }

    pub fn stable_install_path(&self) -> PathBuf {
        if let Ok(explicit) = std::env::var("NEQST_LOK_FALLBACK_INSTALL_PATH") {
            if !explicit.is_empty() {
                return PathBuf::from(explicit);
            }
        }

        self.project_root
            .join("embedded")
            .join("libreoffice")
            .join(self.current_stable_version())
            .join("program")
    }

    pub async fn verify_embedded_hash(&self, install_path: &Path) -> Result<(), String> {
        let expected = self.matrix.embedded_hash_blake3.trim();
        if expected.is_empty() || expected == "REPLACE_WITH_VERIFIED_BLAKE3_OF_LIBREOFFICEKIT_BINARY" {
            return Ok(());
        }

        let lib_path = locate_lok_binary(install_path);
        let Ok(data) = fs::read(&lib_path).await else {
            return Err("lok_binary_not_found".to_string());
        };

        let got = blake3::hash(&data).to_hex().to_string();
        if got != expected {
            return Err("lok_hash_mismatch".to_string());
        }
        Ok(())
    }

    pub fn apply_install_path_env(&self, install_path: &Path) {
        std::env::set_var("NEQST_LOK_INSTALL_PATH", install_path.to_string_lossy().to_string());
        std::env::set_var("NEQST_LOK_LIBRARY_PATH", locate_lok_binary(install_path).to_string_lossy().to_string());
    }

    pub async fn rollback_to_stable(&self, reason: &str, detail: &str) {
        let stable = self.stable_install_path();
        self.apply_install_path_env(&stable);
        self.forensic_dump(
            "lok_rollback_to_stable",
            serde_json::json!({
                "reason": reason,
                "detail": detail,
                "stable_install_path": stable.to_string_lossy(),
            }),
        )
        .await;
    }

    pub async fn forensic_dump(&self, kind: &str, data: serde_json::Value) {
        let dir = std::env::var("NEQST_FORENSIC_DIR")
            .ok()
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("../ForensicData"));

        let _ = fs::create_dir_all(&dir).await;
        let path = dir.join("neqst_forensics.ndjson");

        let event = serde_json::json!({
            "ts_unix": crate::network::unix_now(),
            "kind": kind,
            "data": data
        });

        if let Ok(mut f) = fs::OpenOptions::new().create(true).append(true).open(path).await {
            let _ = f.write_all(event.to_string().as_bytes()).await;
            let _ = f.write_all(b"\n").await;
        }
    }
}

fn resolve_project_root() -> Option<PathBuf> {
    let mut dir = std::env::current_dir().ok()?;
    for _ in 0..12 {
        if dir.join("core").join("compatibility_matrix.json").is_file()
            || dir.join("config").join("compatibility_matrix.json").is_file()
        {
            return Some(dir);
        }
        if !dir.pop() {
            break;
        }
    }
    None
}

fn locate_lok_binary(install_path: &Path) -> PathBuf {
    if cfg!(target_os = "windows") {
        return install_path.join("libreofficekit.dll");
    }
    if cfg!(target_os = "macos") {
        return install_path.join("libreofficekit.dylib");
    }
    install_path.join("libreofficekit.so")
}

