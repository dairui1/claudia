pub mod manager;
pub mod session;
pub mod git_worktree;
pub mod process;
pub mod auto_yes;

pub use manager::SessionManager;
pub use session::{Session, SessionStatus, SessionConfig};
pub use git_worktree::GitWorktree;

use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum SessionEvent {
    StatusChanged { session_id: String, status: SessionStatus },
    OutputAppended { session_id: String, output: String },
    DiffUpdated { session_id: String, stats: DiffStats },
    SessionCreated { session_id: String },
    SessionTerminated { session_id: String },
    Error { session_id: String, error: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffStats {
    pub files_changed: usize,
    pub insertions: usize,
    pub deletions: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub id: String,
    pub project_id: String,
    pub project_path: String,
    pub worktree_path: String,
    pub branch_name: String,
    pub status: SessionStatus,
    pub created_at: String,
    pub updated_at: String,
    pub auto_yes: bool,
    pub output_preview: String,
    pub diff_stats: Option<DiffStats>,
}

pub type EventReceiver = broadcast::Receiver<SessionEvent>;