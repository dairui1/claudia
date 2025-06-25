use std::time::Duration;
use tokio::time::interval;
use tokio::sync::broadcast;
use regex::Regex;
use crate::multi_session::{SessionManager, SessionStatus};

pub struct AutoYesManager {
    patterns: Vec<PromptPattern>,
    poll_interval: Duration,
}

struct PromptPattern {
    regex: Regex,
    response: String,
    description: String,
}

impl Default for AutoYesManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AutoYesManager {
    pub fn new() -> Self {
        let patterns = vec![
            PromptPattern {
                regex: Regex::new(r"(?i)(continue|proceed|yes/no|y/n)\s*[?:]?\s*$").unwrap(),
                response: "yes".to_string(),
                description: "General confirmation prompts".to_string(),
            },
            PromptPattern {
                regex: Regex::new(r"(?i)press enter to continue").unwrap(),
                response: "".to_string(),
                description: "Press enter prompts".to_string(),
            },
            PromptPattern {
                regex: Regex::new(r"(?i)would you like to").unwrap(),
                response: "yes".to_string(),
                description: "Would you like prompts".to_string(),
            },
            PromptPattern {
                regex: Regex::new(r"(?i)is this correct").unwrap(),
                response: "yes".to_string(),
                description: "Confirmation prompts".to_string(),
            },
        ];
        
        Self {
            patterns,
            poll_interval: Duration::from_secs(2),
        }
    }
    
    pub fn add_pattern(&mut self, pattern: &str, response: &str, description: &str) -> Result<(), regex::Error> {
        let regex = Regex::new(pattern)?;
        self.patterns.push(PromptPattern {
            regex,
            response: response.to_string(),
            description: description.to_string(),
        });
        Ok(())
    }
    
    pub async fn start_monitoring(
        &self,
        manager: SessionManager,
        mut shutdown_rx: broadcast::Receiver<()>,
    ) {
        let mut ticker = interval(self.poll_interval);
        
        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    self.check_all_sessions(&manager).await;
                }
                _ = shutdown_rx.recv() => {
                    break;
                }
            }
        }
    }
    
    async fn check_all_sessions(&self, manager: &SessionManager) {
        let sessions = manager.list_active_sessions().await;
        
        for session_info in sessions {
            if !session_info.auto_yes {
                continue;
            }
            
            if session_info.status != SessionStatus::Ready {
                continue;
            }
            
            // Check if the session is waiting for input
            if let Some(prompt) = self.detect_prompt(&session_info.output_preview) {
                if let Err(e) = manager.send_input(&session_info.id, &prompt.response).await {
                    eprintln!("Failed to send auto-yes response: {}", e);
                }
            }
        }
    }
    
    fn detect_prompt(&self, output: &str) -> Option<&PromptPattern> {
        let lines: Vec<&str> = output.lines().collect();
        if lines.is_empty() {
            return None;
        }
        
        // Check the last few lines for prompts
        let recent_lines = lines.iter().rev().take(5).collect::<Vec<_>>();
        
        for line in recent_lines {
            for pattern in &self.patterns {
                if pattern.regex.is_match(line) {
                    return Some(pattern);
                }
            }
        }
        
        None
    }
    
    pub fn is_safe_prompt(output: &str) -> bool {
        // Check for dangerous operations that should not be auto-confirmed
        let dangerous_patterns = vec![
            r"(?i)delete",
            r"(?i)remove",
            r"(?i)force",
            r"(?i)overwrite",
            r"(?i)destructive",
            r"(?i)permanent",
            r"(?i)cannot be undone",
            r"(?i)are you sure",
        ];
        
        for pattern in dangerous_patterns {
            if let Ok(regex) = Regex::new(pattern) {
                if regex.is_match(output) {
                    return false;
                }
            }
        }
        
        true
    }
}