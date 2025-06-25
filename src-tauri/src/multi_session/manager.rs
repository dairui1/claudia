use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock, broadcast};
use anyhow::{Result, Context, bail};
use crate::Database;
use super::{
    Session, SessionConfig, SessionEvent, SessionInfo, SessionStatus,
    GitWorktree, process::ProcessManager, auto_yes::AutoYesManager,
    DiffStats,
};

pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<String, Arc<Session>>>>,
    db: Arc<Database>,
    event_tx: broadcast::Sender<SessionEvent>,
    event_rx: broadcast::Receiver<SessionEvent>,
    auto_yes_manager: Arc<AutoYesManager>,
    max_concurrent_sessions: usize,
}

impl SessionManager {
    pub fn new(db: Arc<Database>, max_concurrent_sessions: usize) -> Self {
        let (event_tx, event_rx) = broadcast::channel(1000);
        
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            db,
            event_tx,
            event_rx,
            auto_yes_manager: Arc::new(AutoYesManager::new()),
            max_concurrent_sessions,
        }
    }
    
    pub fn subscribe_events(&self) -> broadcast::Receiver<SessionEvent> {
        self.event_tx.subscribe()
    }
    
    pub async fn create_session(
        &self,
        project_id: String,
        project_path: PathBuf,
        config: SessionConfig,
    ) -> Result<String> {
        // Check session limit
        let session_count = self.sessions.read().await.len();
        if session_count >= self.max_concurrent_sessions {
            bail!("Maximum concurrent sessions ({}) reached", self.max_concurrent_sessions);
        }
        
        // Create session
        let session = Session::new(
            project_id.clone(),
            project_path.clone(),
            PathBuf::new(), // Will be set after worktree creation
            String::new(),  // Will be set after worktree creation
            config,
        );
        
        let session_id = session.id.clone();
        
        // Create git worktree
        let worktree = GitWorktree::new(
            project_path,
            &session_id,
            &session.config.branch_prefix,
        )?;
        
        worktree.create()
            .context("Failed to create git worktree")?;
        
        // Update session with worktree info
        let session = Arc::new(Session {
            worktree_path: worktree.worktree_path.clone(),
            branch_name: worktree.branch_name.clone(),
            ..session
        });
        
        // Store in database
        self.store_session_in_db(&session).await?;
        
        // Start Claude process
        let mut child = ProcessManager::spawn_claude_session(
            &session,
            self.event_tx.clone(),
        ).await?;
        
        // Store process handle
        *session.process.lock().await = Some(child);
        session.set_status(SessionStatus::Running).await;
        
        // Add to active sessions
        self.sessions.write().await.insert(session_id.clone(), session.clone());
        
        // Send creation event
        let _ = self.event_tx.send(SessionEvent::SessionCreated {
            session_id: session_id.clone(),
        });
        
        Ok(session_id)
    }
    
    pub async fn terminate_session(&self, session_id: &str) -> Result<()> {
        let session = {
            let mut sessions = self.sessions.write().await;
            sessions.remove(session_id)
                .context("Session not found")?
        };
        
        // Terminate the process
        session.terminate().await;
        
        // Remove git worktree
        let worktree = GitWorktree {
            repo_path: session.project_path.clone(),
            worktree_path: session.worktree_path.clone(),
            branch_name: session.branch_name.clone(),
        };
        worktree.remove()?;
        
        // Update database
        self.update_session_status_in_db(session_id, SessionStatus::Terminated).await?;
        
        // Send termination event
        let _ = self.event_tx.send(SessionEvent::SessionTerminated {
            session_id: session_id.to_string(),
        });
        
        Ok(())
    }
    
    pub async fn pause_session(&self, session_id: &str) -> Result<()> {
        let sessions = self.sessions.read().await;
        let session = sessions.get(session_id)
            .context("Session not found")?;
        
        // Commit any pending changes
        let worktree = GitWorktree {
            repo_path: session.project_path.clone(),
            worktree_path: session.worktree_path.clone(),
            branch_name: session.branch_name.clone(),
        };
        worktree.commit_changes("WIP: Pausing session")?;
        
        // Terminate the process but keep the session
        if let Some(mut process) = session.process.lock().await.take() {
            let _ = process.kill().await;
        }
        
        session.set_status(SessionStatus::Paused).await;
        self.update_session_status_in_db(session_id, SessionStatus::Paused).await?;
        
        Ok(())
    }
    
    pub async fn resume_session(&self, session_id: &str) -> Result<()> {
        let sessions = self.sessions.read().await;
        let session = sessions.get(session_id)
            .context("Session not found")?;
        
        if *session.status.lock().await != SessionStatus::Paused {
            bail!("Session is not paused");
        }
        
        // Restart Claude process
        let mut child = ProcessManager::spawn_claude_session(
            session,
            self.event_tx.clone(),
        ).await?;
        
        *session.process.lock().await = Some(child);
        session.set_status(SessionStatus::Running).await;
        self.update_session_status_in_db(session_id, SessionStatus::Running).await?;
        
        Ok(())
    }
    
    pub async fn send_input(&self, session_id: &str, input: &str) -> Result<()> {
        let sessions = self.sessions.read().await;
        let session = sessions.get(session_id)
            .context("Session not found")?;
        
        let mut process_guard = session.process.lock().await;
        if let Some(child) = process_guard.as_mut() {
            ProcessManager::send_input(child, input).await?;
        } else {
            bail!("Session process not running");
        }
        
        Ok(())
    }
    
    pub async fn get_session_output(&self, session_id: &str, lines: usize) -> Result<Vec<String>> {
        let sessions = self.sessions.read().await;
        let session = sessions.get(session_id)
            .context("Session not found")?;
        
        Ok(session.get_output_preview(lines).await)
    }
    
    pub async fn get_session_diff(&self, session_id: &str) -> Result<DiffStats> {
        let sessions = self.sessions.read().await;
        let session = sessions.get(session_id)
            .context("Session not found")?;
        
        let worktree = GitWorktree {
            repo_path: session.project_path.clone(),
            worktree_path: session.worktree_path.clone(),
            branch_name: session.branch_name.clone(),
        };
        
        worktree.get_diff_stats()
    }
    
    pub async fn list_active_sessions(&self) -> Vec<SessionInfo> {
        let sessions = self.sessions.read().await;
        let mut infos = Vec::new();
        
        for session in sessions.values() {
            let diff_stats = self.get_session_diff(&session.id).await.ok();
            infos.push(session.to_info(diff_stats).await);
        }
        
        infos.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        infos
    }
    
    pub async fn update_session_config(
        &self,
        session_id: &str,
        config: SessionConfig,
    ) -> Result<()> {
        let sessions = self.sessions.read().await;
        let session = sessions.get(session_id)
            .context("Session not found")?;
        
        // For now, we can only update auto_yes without restarting
        // Full config update would require session restart
        let mut current_config = session.config.clone();
        current_config.auto_yes = config.auto_yes;
        
        // Update in database
        sqlx::query!(
            "UPDATE multi_sessions SET auto_yes = ?, updated_at = datetime('now') WHERE id = ?",
            config.auto_yes,
            session_id
        )
        .execute(&*self.db.pool)
        .await?;
        
        Ok(())
    }
    
    // Database operations
    async fn store_session_in_db(&self, session: &Session) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO multi_sessions (
                id, project_id, worktree_path, branch_name, status,
                created_at, updated_at, auto_yes, output_log
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            session.id,
            session.project_id,
            session.worktree_path.to_str(),
            session.branch_name,
            "running",
            session.created_at.to_rfc3339(),
            session.created_at.to_rfc3339(),
            session.config.auto_yes,
            ""
        )
        .execute(&*self.db.pool)
        .await?;
        
        Ok(())
    }
    
    async fn update_session_status_in_db(
        &self,
        session_id: &str,
        status: SessionStatus,
    ) -> Result<()> {
        let status_str = serde_json::to_string(&status)?;
        
        sqlx::query!(
            "UPDATE multi_sessions SET status = ?, updated_at = datetime('now') WHERE id = ?",
            status_str,
            session_id
        )
        .execute(&*self.db.pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn start_auto_yes_daemon(&self) {
        let manager = self.clone();
        let shutdown_rx = self.event_tx.subscribe();
        let auto_yes_manager = self.auto_yes_manager.clone();
        
        tokio::spawn(async move {
            auto_yes_manager.start_monitoring(manager, shutdown_rx).await;
        });
    }
}

// Implement Clone manually to handle broadcast receiver
impl Clone for SessionManager {
    fn clone(&self) -> Self {
        Self {
            sessions: self.sessions.clone(),
            db: self.db.clone(),
            event_tx: self.event_tx.clone(),
            event_rx: self.event_tx.subscribe(),
            auto_yes_manager: self.auto_yes_manager.clone(),
            max_concurrent_sessions: self.max_concurrent_sessions,
        }
    }
}