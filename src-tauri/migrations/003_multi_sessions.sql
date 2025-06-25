-- Create multi_sessions table
CREATE TABLE IF NOT EXISTS multi_sessions (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    worktree_path TEXT NOT NULL,
    branch_name TEXT NOT NULL,
    status TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL,
    updated_at TIMESTAMP NOT NULL,
    auto_yes BOOLEAN DEFAULT FALSE,
    output_log TEXT,
    FOREIGN KEY (project_id) REFERENCES projects(id)
);

-- Create session_checkpoints table for pause/resume functionality
CREATE TABLE IF NOT EXISTS session_checkpoints (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    checkpoint_data BLOB NOT NULL,
    created_at TIMESTAMP NOT NULL,
    FOREIGN KEY (session_id) REFERENCES multi_sessions(id) ON DELETE CASCADE
);

-- Create indexes for better performance
CREATE INDEX idx_multi_sessions_project_id ON multi_sessions(project_id);
CREATE INDEX idx_multi_sessions_status ON multi_sessions(status);
CREATE INDEX idx_multi_sessions_created_at ON multi_sessions(created_at);
CREATE INDEX idx_session_checkpoints_session_id ON session_checkpoints(session_id);