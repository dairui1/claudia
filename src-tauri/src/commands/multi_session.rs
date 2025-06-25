use tauri::{AppHandle, Manager, State};
use serde_json::json;
use crate::multi_session::{SessionManager, SessionConfig, SessionInfo, DiffStats};
use crate::Database;
use std::sync::Arc;
use std::path::PathBuf;

#[tauri::command]
pub async fn create_multi_session(
    app: AppHandle,
    project_id: String,
    config: SessionConfig,
    session_manager: State<'_, Arc<SessionManager>>,
) -> Result<String, String> {
    // Get project path from database
    let db = app.state::<Arc<Database>>();
    let project = sqlx::query!(
        "SELECT path FROM projects WHERE id = ?",
        project_id
    )
    .fetch_one(&*db.pool)
    .await
    .map_err(|e| format!("Failed to fetch project: {}", e))?;
    
    let project_path = PathBuf::from(project.path);
    
    session_manager
        .create_session(project_id, project_path, config)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_active_sessions(
    session_manager: State<'_, Arc<SessionManager>>,
) -> Result<Vec<SessionInfo>, String> {
    Ok(session_manager.list_active_sessions().await)
}

#[tauri::command]
pub async fn terminate_session(
    session_id: String,
    session_manager: State<'_, Arc<SessionManager>>,
) -> Result<(), String> {
    session_manager
        .terminate_session(&session_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn pause_session(
    session_id: String,
    session_manager: State<'_, Arc<SessionManager>>,
) -> Result<(), String> {
    session_manager
        .pause_session(&session_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn resume_session(
    session_id: String,
    session_manager: State<'_, Arc<SessionManager>>,
) -> Result<(), String> {
    session_manager
        .resume_session(&session_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn send_input(
    session_id: String,
    input: String,
    session_manager: State<'_, Arc<SessionManager>>,
) -> Result<(), String> {
    session_manager
        .send_input(&session_id, &input)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_session_output(
    session_id: String,
    lines: usize,
    session_manager: State<'_, Arc<SessionManager>>,
) -> Result<Vec<String>, String> {
    session_manager
        .get_session_output(&session_id, lines)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_session_diff(
    session_id: String,
    session_manager: State<'_, Arc<SessionManager>>,
) -> Result<DiffStats, String> {
    session_manager
        .get_session_diff(&session_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_session_config(
    session_id: String,
    config: SessionConfig,
    session_manager: State<'_, Arc<SessionManager>>,
) -> Result<(), String> {
    session_manager
        .update_session_config(&session_id, config)
        .await
        .map_err(|e| e.to_string())
}

// Setup function to initialize the session event forwarding
pub fn setup_session_events(app: &AppHandle, session_manager: Arc<SessionManager>) {
    let app_handle = app.clone();
    let mut event_rx = session_manager.subscribe_events();
    
    tauri::async_runtime::spawn(async move {
        while let Ok(event) = event_rx.recv().await {
            let _ = app_handle.emit("session-event", &event);
        }
    });
}