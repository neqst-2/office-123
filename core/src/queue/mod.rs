use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};

pub mod worker;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum OfficeTask {
    ProcessDocument { path: String, link_ctx: Option<String> },
    BulkEncryptMails { mail_ids: Vec<String> },
    NetworkSyncDelta { peer_id: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TaskProgress {
    Started { task_id: String },
    Processing {
        task_id: String,
        percentage: u8,
        status_message: String,
    },
    Completed { task_id: String, result_summary: String },
    Failed { task_id: String, error_message: String },
}

#[derive(Clone, Debug)]
pub struct TaskEnvelope {
    pub session_id: String,
    pub task_id: String,
    pub task: OfficeTask,
}

pub fn new_task_id() -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let t = crate::network::unix_now();
    format!("task-{}-{}", t, n)
}

pub fn new_session_id() -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let t = crate::network::unix_now();
    format!("sess-{}-{}", t, n)
}

