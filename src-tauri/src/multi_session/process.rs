
use std::process::Stdio;
use tokio::process::{Command, Child};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::broadcast;
use anyhow::{Result, Context};
use crate::multi_session::{Session, SessionEvent, SessionStatus};

pub struct ProcessManager;

impl ProcessManager {
    pub async fn spawn_claude_session(
        session: &Session,
        event_tx: broadcast::Sender<SessionEvent>,
    ) -> Result<Child> {
        let mut cmd = Command::new("claude");
        
        // Set working directory
        let working_dir = session.config.working_directory
            .as_ref()
            .unwrap_or(&session.worktree_path);
        cmd.current_dir(working_dir);
        
        // Add any additional arguments
        for arg in &session.config.claude_args {
            cmd.arg(arg);
        }
        
        // Set environment variables
        for (key, value) in &session.config.environment_vars {
            cmd.env(key, value);
        }
        
        // Configure stdio
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        
        // Spawn the process
        let mut child = cmd.spawn()
            .context("Failed to spawn Claude process")?;
        
        // Set up output monitoring
        if let Some(stdout) = child.stdout.take() {
            let session_id = session.id.clone();
            let session_clone = session.clone();
            let tx = event_tx.clone();
            
            tokio::spawn(async move {
                let reader = BufReader::new(stdout);
                let mut lines = reader.lines();
                
                while let Ok(Some(line)) = lines.next_line().await {
                    // Append to session output
                    session_clone.append_output(line.clone()).await;
                    
                    // Detect status changes from output
                    if let Some(status) = Self::detect_status_from_output(&line) {
                        session_clone.set_status(status.clone()).await;
                        let _ = tx.send(SessionEvent::StatusChanged {
                            session_id: session_id.clone(),
                            status,
                        });
                    }
                    
                    // Send output event
                    let _ = tx.send(SessionEvent::OutputAppended {
                        session_id: session_id.clone(),
                        output: line,
                    });
                }
            });
        }
        
        // Set up stderr monitoring
        if let Some(stderr) = child.stderr.take() {
            let session_id = session.id.clone();
            let session_clone = session.clone();
            let tx = event_tx.clone();
            
            tokio::spawn(async move {
                let reader = BufReader::new(stderr);
                let mut lines = reader.lines();
                
                while let Ok(Some(line)) = lines.next_line().await {
                    // Treat stderr as potential error
                    session_clone.append_output(format!("[ERROR] {}", line)).await;
                    
                    // Check for critical errors
                    if Self::is_critical_error(&line) {
                        session_clone.set_error(line.clone()).await;
                        let _ = tx.send(SessionEvent::Error {
                            session_id: session_id.clone(),
                            error: line.clone(),
                        });
                    }
                }
            });
        }
        
        Ok(child)
    }
    
    fn detect_status_from_output(line: &str) -> Option<SessionStatus> {
        // Pattern matching for Claude status indicators
        if line.contains("Ready") || line.contains("Human:") {
            Some(SessionStatus::Ready)
        } else if line.contains("Working") || line.contains("Thinking") {
            Some(SessionStatus::Loading)
        } else if line.contains("Running") || line.contains("Executing") {
            Some(SessionStatus::Running)
        } else if line.contains("Complete") || line.contains("Done") {
            Some(SessionStatus::Completed)
        } else {
            None
        }
    }
    
    fn is_critical_error(line: &str) -> bool {
        line.contains("FATAL") || 
        line.contains("CRITICAL") ||
        line.contains("Failed to initialize") ||
        line.contains("Permission denied")
    }
    
    pub async fn send_input(child: &mut Child, input: &str) -> Result<()> {
        if let Some(stdin) = child.stdin.as_mut() {
            use tokio::io::AsyncWriteExt;
            stdin.write_all(input.as_bytes()).await
                .context("Failed to write to process stdin")?;
            stdin.write_all(b"\n").await
                .context("Failed to write newline to process stdin")?;
            stdin.flush().await
                .context("Failed to flush process stdin")?;
        }
        Ok(())
    }
    
    pub async fn check_process_health(child: &mut Child) -> bool {
        matches!(child.try_wait(), Ok(None))
    }
}