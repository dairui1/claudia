use std::path::PathBuf;
use std::process::Command;
use anyhow::{Result, Context, bail};
use regex::Regex;

pub struct GitWorktree {
    pub repo_path: PathBuf,
    pub worktree_path: PathBuf,
    pub branch_name: String,
}

impl GitWorktree {
    pub fn new(repo_path: PathBuf, session_id: &str, branch_prefix: &str) -> Result<Self> {
        let branch_name = format!("{}-{}", branch_prefix, &session_id[..8]);
        let worktree_name = format!("session-{}", &session_id[..8]);
        let worktree_path = repo_path.parent()
            .unwrap_or(&repo_path)
            .join(".claudia-worktrees")
            .join(&worktree_name);
        
        Ok(Self {
            repo_path,
            worktree_path,
            branch_name,
        })
    }

    pub fn create(&self) -> Result<()> {
        // Check if repo is a git repository
        if !self.is_git_repo()? {
            bail!("Not a git repository: {:?}", self.repo_path);
        }

        // Get the current branch to base the new branch on
        let base_branch = self.get_current_branch()?;

        // Create the worktree directory if it doesn't exist
        if let Some(parent) = self.worktree_path.parent() {
            std::fs::create_dir_all(parent)
                .context("Failed to create worktree parent directory")?;
        }

        // Create new branch and worktree
        let output = Command::new("git")
            .current_dir(&self.repo_path)
            .args(&[
                "worktree",
                "add",
                "-b",
                &self.branch_name,
                self.worktree_path.to_str().unwrap(),
                &base_branch,
            ])
            .output()
            .context("Failed to create git worktree")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("Failed to create worktree: {}", stderr);
        }

        Ok(())
    }

    pub fn remove(&self) -> Result<()> {
        // Remove the worktree
        let output = Command::new("git")
            .current_dir(&self.repo_path)
            .args(&["worktree", "remove", "--force", self.worktree_path.to_str().unwrap()])
            .output()
            .context("Failed to remove git worktree")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            // Ignore if worktree doesn't exist
            if !stderr.contains("not a working tree") {
                bail!("Failed to remove worktree: {}", stderr);
            }
        }

        // Delete the branch
        let _ = Command::new("git")
            .current_dir(&self.repo_path)
            .args(&["branch", "-D", &self.branch_name])
            .output();

        Ok(())
    }

    pub fn get_diff_stats(&self) -> Result<super::DiffStats> {
        let output = Command::new("git")
            .current_dir(&self.worktree_path)
            .args(&["diff", "--stat", "--no-color"])
            .output()
            .context("Failed to get git diff stats")?;

        if !output.status.success() {
            // Return empty stats if diff fails
            return Ok(super::DiffStats {
                files_changed: 0,
                insertions: 0,
                deletions: 0,
            });
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        self.parse_diff_stats(&stdout)
    }

    pub fn commit_changes(&self, message: &str) -> Result<()> {
        // Stage all changes
        Command::new("git")
            .current_dir(&self.worktree_path)
            .args(&["add", "-A"])
            .output()
            .context("Failed to stage changes")?;

        // Commit
        let output = Command::new("git")
            .current_dir(&self.worktree_path)
            .args(&["commit", "-m", message])
            .output()
            .context("Failed to commit changes")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("nothing to commit") {
                return Ok(());
            }
            bail!("Failed to commit: {}", stderr);
        }

        Ok(())
    }

    fn is_git_repo(&self) -> Result<bool> {
        let output = Command::new("git")
            .current_dir(&self.repo_path)
            .args(&["rev-parse", "--git-dir"])
            .output()
            .context("Failed to check if directory is a git repo")?;

        Ok(output.status.success())
    }

    fn get_current_branch(&self) -> Result<String> {
        let output = Command::new("git")
            .current_dir(&self.repo_path)
            .args(&["rev-parse", "--abbrev-ref", "HEAD"])
            .output()
            .context("Failed to get current branch")?;

        if !output.status.success() {
            bail!("Failed to get current branch");
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    fn parse_diff_stats(&self, output: &str) -> Result<super::DiffStats> {
        let mut stats = super::DiffStats {
            files_changed: 0,
            insertions: 0,
            deletions: 0,
        };

        // Parse the summary line like "5 files changed, 123 insertions(+), 45 deletions(-)"
        let re = Regex::new(r"(\d+) files? changed(?:, (\d+) insertions?\(\+\))?(?:, (\d+) deletions?\(-\))?")?;
        
        for line in output.lines().rev() {
            if let Some(captures) = re.captures(line) {
                if let Some(files) = captures.get(1) {
                    stats.files_changed = files.as_str().parse().unwrap_or(0);
                }
                if let Some(insertions) = captures.get(2) {
                    stats.insertions = insertions.as_str().parse().unwrap_or(0);
                }
                if let Some(deletions) = captures.get(3) {
                    stats.deletions = deletions.as_str().parse().unwrap_or(0);
                }
                break;
            }
        }

        Ok(stats)
    }
}

impl Drop for GitWorktree {
    fn drop(&mut self) {
        // Best effort cleanup
        let _ = self.remove();
    }
}