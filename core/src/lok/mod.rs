<<<<<<< HEAD
use std::{
    ffi::{CStr, CString},
    path::{Path, PathBuf},
    sync::{Arc, OnceLock},
};

use libloading::Library;
use serde_json::json;
use tokio::fs;

use crate::orchestrator::Db;

pub mod bindings;
pub mod lifecycle;

#[derive(Clone)]
pub struct DocumentMetaExtract {
    pub record_id: Option<String>,
    pub filename: String,
    pub file_size: u64,
    pub storage_path: String,
    pub mime_type: String,
    pub parts: i32,
    pub structure_hash: String,
    pub is_graph_enhanced: bool,
    pub lok_available: bool,
}

pub async fn parse_and_store_document_meta(
    db: &Db,
    file_path: &str,
) -> Result<DocumentMetaExtract, Box<dyn std::error::Error + Send + Sync>> {
    let storage_path = file_path.to_string();
    let abs_path = resolve_to_absolute(file_path)?;
    let meta = fs::metadata(&abs_path).await?;
    let file_size = meta.len();
    let filename = abs_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(file_path)
        .to_string();

    let mime_type = guess_mime(&abs_path);
    let is_graph_enhanced = is_odf(&abs_path);

    let (parts, lok_available) = match try_get_parts_via_lok(&abs_path).await {
        Ok(parts) => (parts, true),
        Err(_) => (0, false),
    };

    let structure_hash = hash_file_prefix(&abs_path).await.unwrap_or_else(|_| {
        let s = format!("{}:{}", filename, file_size);
        blake3::hash(s.as_bytes()).to_hex().to_string()
    });

    let sql = r#"
UPSERT document_meta SET
  filename = $filename,
  file_size = $file_size,
  storage_path = $storage_path,
  mime_type = $mime_type,
  last_modified = time::now()
WHERE storage_path = $storage_path;
SELECT * FROM document_meta WHERE storage_path = $storage_path LIMIT 1;
"#;

    let mut resp = db
        .query_bind(
            sql,
            vec![
                ("filename".to_string(), surrealdb::sql::Value::from(filename.as_str())),
                ("file_size".to_string(), surrealdb::sql::Value::from(file_size as i64)),
                ("storage_path".to_string(), surrealdb::sql::Value::from(storage_path.as_str())),
                ("mime_type".to_string(), surrealdb::sql::Value::from(mime_type.as_str())),
            ],
        )
        .await?;

    let records: Vec<serde_json::Value> = resp.take(1).unwrap_or_default();
    let record_id = records
        .first()
        .and_then(|v| v.get("id"))
        .map(|v| v.to_string());

    Ok(DocumentMetaExtract {
        record_id,
        filename,
        file_size,
        storage_path,
        mime_type,
        parts,
        structure_hash,
        is_graph_enhanced,
        lok_available,
    })
}

async fn try_get_parts_via_lok(abs_path: &Path) -> Result<i32, Box<dyn std::error::Error + Send + Sync>> {
    ensure_lok_runtime().await?;
    let runtime = LokRuntime::get()?;
    let url = to_file_url(abs_path)?;
    let url_c = CString::new(url)?;

    unsafe {
        let doc = bindings::office_document_load(runtime.office, url_c.as_ptr());
        if doc.is_null() {
            return Err("lok_document_load_failed".into());
        }
        let parts = bindings::document_get_parts(doc);
        bindings::document_destroy(doc);
        Ok(parts)
    }
}

async fn ensure_lok_runtime() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let manager = lifecycle::CoreVersionManager::load().await?;

    if let Some(candidate) = manager.candidate_install_path() {
        manager.apply_install_path_env(&candidate);
        let candidate_verify = manager.verify_embedded_hash(&candidate).await;
        if candidate_verify.is_err() {
            manager
                .forensic_dump(
                    "lok_candidate_hash_failed",
                    json!({ "candidate_install_path": candidate.to_string_lossy(), "reason": "hash_mismatch_or_missing" }),
                )
                .await;
        }

        let res = std::panic::catch_unwind(|| LokRuntime::get().map(|_| ()));
        match res {
            Ok(Ok(())) => return Ok(()),
            Ok(Err(err)) => {
                manager.rollback_to_stable("candidate_init_failed", &err.to_string()).await;
            }
            Err(_) => {
                manager.rollback_to_stable("candidate_panic", "catch_unwind").await;
            }
        }
    } else {
        let stable = manager.stable_install_path();
        manager.apply_install_path_env(&stable);
        let _ = manager.verify_embedded_hash(&stable).await;
    }

    let res = std::panic::catch_unwind(|| LokRuntime::get().map(|_| ()));
    match res {
        Ok(Ok(())) => Ok(()),
        Ok(Err(err)) => {
            manager
                .forensic_dump(
                    "lok_init_failed_stable",
                    json!({ "detail": err.to_string(), "install_path": std::env::var("NEQST_LOK_INSTALL_PATH").unwrap_or_default() }),
                )
                .await;
            Err(err)
        }
        Err(_) => {
            manager
                .forensic_dump(
                    "lok_panic_stable",
                    json!({ "install_path": std::env::var("NEQST_LOK_INSTALL_PATH").unwrap_or_default() }),
                )
                .await;
            Err("lok_panic".into())
        }
    }
}

fn resolve_to_absolute(file_path: &str) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    let path = PathBuf::from(file_path);
    if path.is_absolute() {
        return Ok(path);
    }
    Ok(std::env::current_dir()?.join(path))
}

fn is_odf(path: &Path) -> bool {
    match path.extension().and_then(|e| e.to_str()).map(|s| s.to_ascii_lowercase()) {
        Some(ext) if ext == "odt" || ext == "ods" || ext == "odp" => true,
        _ => false,
    }
}

fn guess_mime(path: &Path) -> String {
    match path.extension().and_then(|e| e.to_str()).map(|s| s.to_ascii_lowercase()) {
        Some(ext) if ext == "odt" => "application/vnd.oasis.opendocument.text".to_string(),
        Some(ext) if ext == "ods" => "application/vnd.oasis.opendocument.spreadsheet".to_string(),
        Some(ext) if ext == "odp" => "application/vnd.oasis.opendocument.presentation".to_string(),
        Some(ext) if ext == "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document".to_string(),
        Some(ext) if ext == "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet".to_string(),
        _ => "application/octet-stream".to_string(),
    }
}

async fn hash_file_prefix(path: &Path) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    use tokio::io::AsyncReadExt;

    let mut file = fs::File::open(path).await?;
    let mut buf = vec![0u8; 64 * 1024];
    let n = file.read(&mut buf).await?;
    let hash = blake3::hash(&buf[..n]).to_hex().to_string();
    Ok(hash)
}

fn to_file_url(path: &Path) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let s = path
        .canonicalize()
        .unwrap_or_else(|_| path.to_path_buf())
        .to_string_lossy()
        .replace('\\', "/");

    if s.contains(":/") {
        return Ok(format!("file:///{}", s));
    }
    Ok(format!("file://{}", s))
}

struct LokRuntime {
    _lib: Arc<Library>,
    office: *mut bindings::LibreOfficeKit,
}

impl LokRuntime {
    fn get() -> Result<&'static LokRuntime, Box<dyn std::error::Error + Send + Sync>> {
        static INSTANCE: OnceLock<LokRuntime> = OnceLock::new();
        INSTANCE.get_or_try_init(|| init_runtime())
    }
}

fn init_runtime() -> Result<LokRuntime, Box<dyn std::error::Error + Send + Sync>> {
    let install_path = std::env::var("NEQST_LOK_INSTALL_PATH").unwrap_or_default();
    if install_path.is_empty() {
        return Err("NEQST_LOK_INSTALL_PATH_not_set".into());
    }

    let lib = load_lok_library()?;
    let lok_init: libloading::Symbol<bindings::LokInitFn> = unsafe { lib.get(b"lok_init\0")? };

    let install_c = CString::new(install_path)?;
    let office = unsafe { lok_init(install_c.as_ptr()) };
    if office.is_null() {
        return Err(json!({"error": "lok_init_failed"}).to_string().into());
    }

    Ok(LokRuntime {
        _lib: Arc::new(lib),
        office,
    })
}

fn load_lok_library() -> Result<Library, Box<dyn std::error::Error + Send + Sync>> {
    let candidates = if cfg!(target_os = "windows") {
        vec![
            "libreofficekit.dll".to_string(),
            "libreofficekitlo.dll".to_string(),
        ]
    } else if cfg!(target_os = "macos") {
        vec!["libreofficekit.dylib".to_string(), "libreofficekit.1.dylib".to_string()]
    } else {
        vec!["libreofficekit.so".to_string(), "libreofficekit.so.1".to_string()]
    };

    if let Ok(explicit) = std::env::var("NEQST_LOK_LIBRARY_PATH") {
        if !explicit.is_empty() {
            return Ok(unsafe { Library::new(explicit)? });
        }
    }

    for name in candidates {
        if let Ok(lib) = unsafe { Library::new(&name) } {
            return Ok(lib);
        }
    }

    Err("lok_library_not_found".into())
}

pub unsafe fn lok_init_wrapper(path: *const std::ffi::c_char) -> *mut bindings::LibreOfficeKit {
    if path.is_null() {
        return std::ptr::null_mut();
    }
    let install = CStr::from_ptr(path).to_string_lossy().to_string();
    std::env::set_var("NEQST_LOK_INSTALL_PATH", install);
    LokRuntime::get().map(|r| r.office).unwrap_or(std::ptr::null_mut())
}

pub unsafe fn lok_document_load(
    client: *mut bindings::LibreOfficeKit,
    url: *const std::ffi::c_char,
) -> *mut bindings::LibreOfficeKitDocument {
    bindings::office_document_load(client, url)
}

=======
use std::{
    ffi::{CStr, CString},
    path::{Path, PathBuf},
    sync::{Arc, OnceLock},
};

use libloading::Library;
use serde_json::json;
use tokio::fs;

use crate::orchestrator::Db;

pub mod bindings;

#[derive(Clone)]
pub struct DocumentMetaExtract {
    pub record_id: Option<String>,
    pub filename: String,
    pub file_size: u64,
    pub storage_path: String,
    pub mime_type: String,
    pub parts: i32,
    pub structure_hash: String,
    pub is_graph_enhanced: bool,
    pub lok_available: bool,
}

pub async fn parse_and_store_document_meta(
    db: &Db,
    file_path: &str,
) -> Result<DocumentMetaExtract, Box<dyn std::error::Error + Send + Sync>> {
    let storage_path = file_path.to_string();
    let abs_path = resolve_to_absolute(file_path)?;
    let meta = fs::metadata(&abs_path).await?;
    let file_size = meta.len();
    let filename = abs_path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(file_path)
        .to_string();

    let mime_type = guess_mime(&abs_path);
    let is_graph_enhanced = is_odf(&abs_path);

    let (parts, lok_available) = match try_get_parts_via_lok(&abs_path).await {
        Ok(parts) => (parts, true),
        Err(_) => (0, false),
    };

    let structure_hash = hash_file_prefix(&abs_path).await.unwrap_or_else(|_| {
        let s = format!("{}:{}", filename, file_size);
        blake3::hash(s.as_bytes()).to_hex().to_string()
    });

    let sql = r#"
UPSERT document_meta SET
  filename = $filename,
  file_size = $file_size,
  storage_path = $storage_path,
  mime_type = $mime_type,
  last_modified = time::now()
WHERE storage_path = $storage_path;
SELECT * FROM document_meta WHERE storage_path = $storage_path LIMIT 1;
"#;

    let mut resp = db
        .query_bind(
            sql,
            vec![
                ("filename", surrealdb::sql::Value::from(filename.as_str())),
                ("file_size", surrealdb::sql::Value::from(file_size as i64)),
                ("storage_path", surrealdb::sql::Value::from(storage_path.as_str())),
                ("mime_type", surrealdb::sql::Value::from(mime_type.as_str())),
            ],
        )
        .await?;

    let records: Vec<serde_json::Value> = resp.take(1).unwrap_or_default();
    let record_id = records
        .first()
        .and_then(|v| v.get("id"))
        .map(|v| v.to_string());

    Ok(DocumentMetaExtract {
        record_id,
        filename,
        file_size,
        storage_path,
        mime_type,
        parts,
        structure_hash,
        is_graph_enhanced,
        lok_available,
    })
}

async fn try_get_parts_via_lok(abs_path: &Path) -> Result<i32, Box<dyn std::error::Error + Send + Sync>> {
    let runtime = LokRuntime::get()?;
    let url = to_file_url(abs_path)?;
    let url_c = CString::new(url)?;

    unsafe {
        let doc = bindings::office_document_load(runtime.office, url_c.as_ptr());
        if doc.is_null() {
            return Err("lok_document_load_failed".into());
        }
        let parts = bindings::document_get_parts(doc);
        bindings::document_destroy(doc);
        Ok(parts)
    }
}

fn resolve_to_absolute(file_path: &str) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    let path = PathBuf::from(file_path);
    if path.is_absolute() {
        return Ok(path);
    }
    Ok(std::env::current_dir()?.join(path))
}

fn is_odf(path: &Path) -> bool {
    match path.extension().and_then(|e| e.to_str()).map(|s| s.to_ascii_lowercase()) {
        Some(ext) if ext == "odt" || ext == "ods" || ext == "odp" => true,
        _ => false,
    }
}

fn guess_mime(path: &Path) -> String {
    match path.extension().and_then(|e| e.to_str()).map(|s| s.to_ascii_lowercase()) {
        Some(ext) if ext == "odt" => "application/vnd.oasis.opendocument.text".to_string(),
        Some(ext) if ext == "ods" => "application/vnd.oasis.opendocument.spreadsheet".to_string(),
        Some(ext) if ext == "odp" => "application/vnd.oasis.opendocument.presentation".to_string(),
        Some(ext) if ext == "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document".to_string(),
        Some(ext) if ext == "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet".to_string(),
        _ => "application/octet-stream".to_string(),
    }
}

async fn hash_file_prefix(path: &Path) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    use tokio::io::AsyncReadExt;

    let mut file = fs::File::open(path).await?;
    let mut buf = vec![0u8; 64 * 1024];
    let n = file.read(&mut buf).await?;
    let hash = blake3::hash(&buf[..n]).to_hex().to_string();
    Ok(hash)
}

fn to_file_url(path: &Path) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let s = path
        .canonicalize()
        .unwrap_or_else(|_| path.to_path_buf())
        .to_string_lossy()
        .replace('\\', "/");

    if s.contains(":/") {
        return Ok(format!("file:///{}", s));
    }
    Ok(format!("file://{}", s))
}

struct LokRuntime {
    _lib: Arc<Library>,
    office: *mut bindings::LibreOfficeKit,
}

impl LokRuntime {
    fn get() -> Result<&'static LokRuntime, Box<dyn std::error::Error + Send + Sync>> {
        static INSTANCE: OnceLock<LokRuntime> = OnceLock::new();
        INSTANCE.get_or_try_init(|| init_runtime())
    }
}

fn init_runtime() -> Result<LokRuntime, Box<dyn std::error::Error + Send + Sync>> {
    let install_path = std::env::var("NEQST_LOK_INSTALL_PATH").unwrap_or_default();
    if install_path.is_empty() {
        return Err("NEQST_LOK_INSTALL_PATH_not_set".into());
    }

    let lib = load_lok_library()?;
    let lok_init: libloading::Symbol<bindings::LokInitFn> = unsafe { lib.get(b"lok_init\0")? };

    let install_c = CString::new(install_path)?;
    let office = unsafe { lok_init(install_c.as_ptr()) };
    if office.is_null() {
        return Err(json!({"error": "lok_init_failed"}).to_string().into());
    }

    Ok(LokRuntime {
        _lib: Arc::new(lib),
        office,
    })
}

fn load_lok_library() -> Result<Library, Box<dyn std::error::Error + Send + Sync>> {
    let candidates = if cfg!(target_os = "windows") {
        vec![
            "libreofficekit.dll".to_string(),
            "libreofficekitlo.dll".to_string(),
        ]
    } else if cfg!(target_os = "macos") {
        vec!["libreofficekit.dylib".to_string(), "libreofficekit.1.dylib".to_string()]
    } else {
        vec!["libreofficekit.so".to_string(), "libreofficekit.so.1".to_string()]
    };

    if let Ok(explicit) = std::env::var("NEQST_LOK_LIBRARY_PATH") {
        if !explicit.is_empty() {
            return Ok(unsafe { Library::new(explicit)? });
        }
    }

    for name in candidates {
        if let Ok(lib) = unsafe { Library::new(&name) } {
            return Ok(lib);
        }
    }

    Err("lok_library_not_found".into())
}

pub unsafe fn lok_init_wrapper(path: *const std::ffi::c_char) -> *mut bindings::LibreOfficeKit {
    if path.is_null() {
        return std::ptr::null_mut();
    }
    let install = CStr::from_ptr(path).to_string_lossy().to_string();
    std::env::set_var("NEQST_LOK_INSTALL_PATH", install);
    LokRuntime::get().map(|r| r.office).unwrap_or(std::ptr::null_mut())
}

pub unsafe fn lok_document_load(
    client: *mut bindings::LibreOfficeKit,
    url: *const std::ffi::c_char,
) -> *mut bindings::LibreOfficeKitDocument {
    bindings::office_document_load(client, url)
}

>>>>>>> 0dc035f57a1c694c8225272cdbd0bfc9c9d60bb9
