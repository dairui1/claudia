# Multi-Session Integration Guide

This guide explains how to integrate the multi-session management feature into the Claudia application.

## Backend Integration

### 1. Update `src-tauri/src/main.rs`

Add the multi_session module and commands:

```rust
mod multi_session;
mod commands;

use multi_session::SessionManager;
use commands::multi_session as multi_session_commands;
use std::sync::Arc;

fn main() {
    tauri::Builder::default()
        .manage(Arc::new(database))
        .manage(Arc::new(SessionManager::new(Arc::clone(&database), 10)))
        .invoke_handler(tauri::generate_handler![
            // ... existing commands
            multi_session_commands::create_multi_session,
            multi_session_commands::list_active_sessions,
            multi_session_commands::terminate_session,
            multi_session_commands::pause_session,
            multi_session_commands::resume_session,
            multi_session_commands::send_input,
            multi_session_commands::get_session_output,
            multi_session_commands::get_session_diff,
            multi_session_commands::update_session_config,
        ])
        .setup(|app| {
            let session_manager = app.state::<Arc<SessionManager>>().clone();
            
            // Setup event forwarding
            multi_session_commands::setup_session_events(&app.handle(), session_manager.clone());
            
            // Start auto-yes daemon if configured
            if config.auto_yes_enabled {
                session_manager.start_auto_yes_daemon().await;
            }
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### 2. Update `src-tauri/Cargo.toml`

Add required dependencies:

```toml
[dependencies]
# ... existing dependencies
uuid = { version = "1.6", features = ["v4", "serde"] }
regex = "1.10"
```

### 3. Update `src-tauri/src/commands/mod.rs`

Add the multi_session module:

```rust
pub mod agents;
pub mod claude;
pub mod mcp;
pub mod sandbox;
pub mod multi_session; // Add this line
```

## Frontend Integration

### 1. Update App Navigation

Add the Multi-Session Manager to your main navigation in `src/App.tsx`:

```tsx
import MultiSessionDashboard from './components/multi-session/MultiSessionDashboard';

// In your navigation/routing setup:
<Route path="/multi-sessions" element={<MultiSessionDashboard />} />

// In your sidebar/menu:
<NavLink to="/multi-sessions">
  <Layers className="w-4 h-4" />
  Multi-Sessions
</NavLink>
```

### 2. Add to Project Context Menu

In your project list or Claude Code session components, add an option to create a multi-session:

```tsx
const createMultiSession = async (projectId: string) => {
  const config = {
    auto_yes: false,
    max_output_buffer: 10000,
    environment_vars: [],
    working_directory: null,
    branch_prefix: 'claudia-session',
    claude_args: [],
  };
  
  try {
    const sessionId = await invoke('create_multi_session', {
      projectId,
      config,
    });
    navigate(`/multi-sessions?session=${sessionId}`);
  } catch (error) {
    toast.error(`Failed to create session: ${error}`);
  }
};
```

## Database Migration

Run the database migration to create the required tables:

```bash
# The migration file is already created at:
# src-tauri/migrations/003_multi_sessions.sql
```

## Configuration

Add multi-session settings to your app configuration:

```json
{
  "multi_session": {
    "max_concurrent_sessions": 10,
    "auto_yes_enabled": false,
    "default_branch_prefix": "claudia-session",
    "auto_yes_poll_interval_seconds": 2
  }
}
```

## Usage Examples

### Creating a Session Programmatically

```typescript
// From any component
import { invoke } from '@tauri-apps/api/core';

const startSession = async () => {
  const sessionId = await invoke('create_multi_session', {
    projectId: 'your-project-id',
    config: {
      auto_yes: true,
      branch_prefix: 'feature',
      claude_args: ['--help'], // Initial command
    },
  });
};
```

### Listening to Session Events

```typescript
import { listen } from '@tauri-apps/api/event';

useEffect(() => {
  const unlisten = listen('session-event', (event) => {
    console.log('Session event:', event.payload);
    // Handle the event based on type
  });
  
  return () => {
    unlisten.then(fn => fn());
  };
}, []);
```

### Batch Operations

```typescript
// Pause all running sessions
const pauseAll = async () => {
  const sessions = await invoke('list_active_sessions');
  const running = sessions.filter(s => s.status === 'running');
  
  await Promise.all(
    running.map(s => invoke('pause_session', { sessionId: s.id }))
  );
};
```

## Best Practices

1. **Session Limits**: Set reasonable limits for concurrent sessions based on system resources
2. **Auto-Yes Safety**: Always validate prompts before enabling auto-yes mode
3. **Branch Naming**: Use descriptive branch prefixes to identify session purposes
4. **Cleanup**: Regularly clean up terminated sessions to free disk space
5. **Monitoring**: Implement proper logging and monitoring for production use

## Troubleshooting

### Common Issues

1. **Git Worktree Errors**: Ensure the project is a valid Git repository
2. **Process Spawn Failures**: Check that Claude Code is installed and in PATH
3. **Permission Errors**: Ensure the app has write permissions to create worktrees
4. **Resource Exhaustion**: Monitor system resources when running many sessions

### Debug Mode

Enable debug logging for multi-session:

```rust
env_logger::Builder::from_env(Env::default().default_filter_or("multi_session=debug")).init();
```

## Security Considerations

1. **Sandboxing**: Each session inherits Claudia's existing sandbox security
2. **File Access**: Sessions are restricted to their worktree directories
3. **Command Injection**: Input is properly escaped before passing to processes
4. **Auto-Yes Validation**: Dangerous operations are blocked in auto-yes mode