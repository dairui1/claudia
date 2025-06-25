# Multi-Session Agent Management Plan for Claudia

## Overview
This document outlines the implementation plan for adding claude-squad-like multi-session agent management capabilities to Claudia. The goal is to enable users to run multiple AI coding agents concurrently with proper isolation, monitoring, and control.

## Key Features to Implement

### 1. Multi-Session Architecture
- **Concurrent Sessions**: Support running multiple Claude Code instances simultaneously
- **Session Isolation**: Each session runs in its own Git worktree to prevent conflicts
- **Resource Management**: Efficient handling of system resources (CPU, memory, file handles)
- **Session Persistence**: Save and restore session states across application restarts

### 2. Git Worktree Integration
- **Automatic Worktree Creation**: Create isolated Git worktrees for each session
- **Branch Management**: Configurable branch prefix (e.g., `claudia-session-{id}`)
- **Diff Tracking**: Real-time diff statistics for each session
- **Cleanup**: Automatic worktree cleanup on session termination

### 3. Session Status Monitoring
- **Status Types**: Running, Ready, Loading, Paused, Error, Completed
- **Real-time Updates**: WebSocket-based status updates from Rust backend
- **Output Streaming**: Capture and stream session output to UI
- **Progress Tracking**: Monitor task completion within each session

### 4. UI Components
- **Multi-Session Dashboard**: Grid/list view of all active sessions
- **Session Preview**: Live preview of session output and current state
- **Diff Viewer**: Show file changes for each session
- **Control Panel**: Start, pause, resume, terminate controls per session
- **Global Controls**: Batch operations on multiple sessions

### 5. Auto-Yes Mode
- **Background Operation**: Allow sessions to run unattended
- **Smart Prompt Detection**: Identify when Claude is waiting for confirmation
- **Configurable Responses**: Set default responses for common prompts
- **Safety Guards**: Prevent destructive operations in auto mode

### 6. Pause/Resume Functionality
- **State Preservation**: Save complete session state on pause
- **Branch Protection**: Keep Git branches intact during pause
- **Quick Resume**: Restore session exactly where it left off
- **Multi-day Sessions**: Support long-running tasks across restarts

## Technical Architecture

### Backend (Rust/Tauri)

#### New Modules
```
src-tauri/src/
├── multi_session/
│   ├── mod.rs              # Module exports
│   ├── manager.rs          # SessionManager struct
│   ├── session.rs          # Individual session handling
│   ├── git_worktree.rs     # Git worktree operations
│   ├── process.rs          # Process spawning and monitoring
│   └── auto_yes.rs         # Auto-yes functionality
```

#### Key Components

1. **SessionManager**
```rust
pub struct SessionManager {
    sessions: Arc<Mutex<HashMap<String, Session>>>,
    db: Arc<Database>,
    event_tx: broadcast::Sender<SessionEvent>,
}
```

2. **Session**
```rust
pub struct Session {
    id: String,
    project_path: PathBuf,
    worktree_path: PathBuf,
    branch_name: String,
    process: Option<Child>,
    status: SessionStatus,
    output_buffer: VecDeque<String>,
    created_at: DateTime<Utc>,
    auto_yes: bool,
}
```

3. **Git Worktree Management**
```rust
pub struct GitWorktree {
    repo_path: PathBuf,
    worktree_path: PathBuf,
    branch_name: String,
}
```

### Frontend (React/TypeScript)

#### New Components
```
src/components/multi-session/
├── MultiSessionDashboard.tsx   # Main dashboard view
├── SessionCard.tsx             # Individual session display
├── SessionPreview.tsx          # Live output preview
├── SessionDiffView.tsx         # Git diff display
├── SessionControls.tsx         # Control buttons
├── GlobalSessionControls.tsx   # Batch operations
└── AutoYesConfig.tsx          # Auto-yes settings
```

#### State Management
```typescript
interface MultiSessionState {
  sessions: Map<string, SessionInfo>;
  activeSessionId: string | null;
  globalSettings: {
    autoYes: boolean;
    maxConcurrentSessions: number;
    defaultBranchPrefix: string;
  };
}
```

### Database Schema
```sql
-- New tables for multi-session support
CREATE TABLE multi_sessions (
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

CREATE TABLE session_checkpoints (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    checkpoint_data BLOB NOT NULL,
    created_at TIMESTAMP NOT NULL,
    FOREIGN KEY (session_id) REFERENCES multi_sessions(id)
);
```

## Implementation Phases

### Phase 1: Core Infrastructure (Week 1-2)
1. Implement SessionManager in Rust
2. Add Git worktree management
3. Create database schema and migrations
4. Basic process spawning and monitoring

### Phase 2: UI Components (Week 2-3)
1. Create MultiSessionDashboard component
2. Implement session cards with status display
3. Add basic controls (start, stop)
4. Integrate with existing Claudia navigation

### Phase 3: Advanced Features (Week 3-4)
1. Implement auto-yes mode
2. Add pause/resume functionality
3. Create diff viewer component
4. Add session output streaming

### Phase 4: Polish & Testing (Week 4-5)
1. Add comprehensive error handling
2. Implement resource limits
3. Create unit and integration tests
4. Performance optimization

## Integration Points

### With Existing Features
1. **CC Agents**: Multi-session support for agent runs
2. **Timeline**: Each session contributes to project timeline
3. **Checkpoints**: Compatible with existing checkpoint system
4. **Sandbox**: Leverage existing sandboxing for security

### API Endpoints (Tauri Commands)
```typescript
// New commands
invoke('create_multi_session', { projectId, config })
invoke('list_active_sessions', {})
invoke('pause_session', { sessionId })
invoke('resume_session', { sessionId })
invoke('terminate_session', { sessionId })
invoke('get_session_output', { sessionId, lines })
invoke('get_session_diff', { sessionId })
invoke('update_session_config', { sessionId, config })
```

## Security Considerations
1. **Process Isolation**: Each session runs with limited permissions
2. **Resource Limits**: CPU and memory caps per session
3. **File System Access**: Restricted to worktree directory
4. **Network Access**: Configurable per session
5. **Auto-Yes Safety**: Whitelist of safe operations

## Performance Targets
- Support up to 10 concurrent sessions
- < 100ms UI update latency
- < 500ms session creation time
- Minimal CPU overhead when sessions are idle

## Success Metrics
1. Users can run 5+ Claude sessions simultaneously
2. No Git conflicts between sessions
3. Sessions can run for hours unattended
4. Easy pause/resume across app restarts
5. Clear visibility into all session states

## Future Enhancements
1. Session templates and presets
2. Distributed sessions across machines
3. Session sharing and collaboration
4. Advanced scheduling and queuing
5. Integration with CI/CD pipelines