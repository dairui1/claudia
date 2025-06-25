use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::process::Child;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    Initializing,
    Running,
    Ready,
    Loading,
    Paused,
    Error,
    Completed,
    Terminated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    pub auto_yes: bool,
    pub max_output_buffer: usize,
    pub environment_vars: Vec<(String, String)>,
    pub working_directory: Option<PathBuf>,
    pub branch_prefix: String,
    pub claude_args: Vec<String>,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            auto_yes: false,
            max_output_buffer: 10000,
            environment_vars: vec![],
            working_directory: None,
            branch_prefix: "claudia-session".to_string(),
            claude_args: vec![],
        }
    }
}

#[derive(Debug)]
pub struct Session {
    pub id: String,
    pub project_id: String,
    pub project_path: PathBuf,
    pub worktree_path: PathBuf,
    pub branch_name: String,
    pub process: Arc<Mutex<Option<Child>>>,
    pub status: Arc<Mutex<SessionStatus>>,
    pub output_buffer: Arc<Mutex<VecDeque<String>>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Arc<Mutex<DateTime<Utc>>>,
    pub config: SessionConfig,
    pub error_message: Arc<Mutex<Option<String>>>,
}

impl Clone for Session {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            project_id: self.project_id.clone(),
            project_path: self.project_path.clone(),
            worktree_path: self.worktree_path.clone(),
            branch_name: self.branch_name.clone(),
            process: self.process.clone(),
            status: self.status.clone(),
            output_buffer: self.output_buffer.clone(),
            created_at: self.created_at,
            updated_at: self.updated_at.clone(),
            config: self.config.clone(),
            error_message: self.error_message.clone(),
        }
    }
}

impl Session {
    pub fn new(
        project_id: String,
        project_path: PathBuf,
        worktree_path: PathBuf,
        branch_name: String,
        config: SessionConfig,
    ) -> Self {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        
        Self {
            id,
            project_id,
            project_path,
            worktree_path,
            branch_name,
            process: Arc::new(Mutex::new(None)),
            status: Arc::new(Mutex::new(SessionStatus::Initializing)),
            output_buffer: Arc::new(Mutex::new(VecDeque::with_capacity(config.max_output_buffer))),
            created_at: now,
            updated_at: Arc::new(Mutex::new(now)),
            config,
            error_message: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn append_output(&self, line: String) {
        let mut buffer = self.output_buffer.lock().await;
        if buffer.len() >= self.config.max_output_buffer {
            buffer.pop_front();
        }
        buffer.push_back(line);
        
        let mut updated_at = self.updated_at.lock().await;
        *updated_at = Utc::now();
    }

    pub async fn get_output_preview(&self, lines: usize) -> Vec<String> {
        let buffer = self.output_buffer.lock().await;
        buffer.iter()
            .rev()
            .take(lines)
            .rev()
            .cloned()
            .collect()
    }

    pub async fn set_status(&self, status: SessionStatus) {
        let mut current_status = self.status.lock().await;
        *current_status = status;
        
        let mut updated_at = self.updated_at.lock().await;
        *updated_at = Utc::now();
    }

    pub async fn set_error(&self, error: String) {
        let mut error_message = self.error_message.lock().await;
        *error_message = Some(error);
        self.set_status(SessionStatus::Error).await;
    }

    pub async fn terminate(&self) {
        if let Some(mut process) = self.process.lock().await.take() {
            let _ = process.kill().await;
        }
        self.set_status(SessionStatus::Terminated).await;
    }

    pub async fn to_info(&self, diff_stats: Option<super::DiffStats>) -> super::SessionInfo {
        let status = self.status.lock().await.clone();
        let output_preview = self.get_output_preview(5).await.join("\n");
        let updated_at = self.updated_at.lock().await;
        
        super::SessionInfo {
            id: self.id.clone(),
            project_id: self.project_id.clone(),
            project_path: self.project_path.display().to_string(),
            worktree_path: self.worktree_path.display().to_string(),
            branch_name: self.branch_name.clone(),
            status,
            created_at: self.created_at.to_rfc3339(),
            updated_at: updated_at.to_rfc3339(),
            auto_yes: self.config.auto_yes,
            output_preview,
            diff_stats,
        }
    }
}