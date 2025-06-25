use tauri::State;
use crate::multi_session::{SessionManager, SessionConfig, SessionInfo, DiffStats};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tauri::command]
pub async fn create_multi_session(
    session_manager: State<'_, Arc<Mutex<SessionManager>>>,
    project_id: String,
    project_path: String,
    config: SessionConfig,
) -> Result<String, String> {
    let manager = session_manager.lock().await;
    manager
        .create_session(project_id, project_path.into(), config)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_active_sessions(
    session_manager: State<'_, Arc<Mutex<SessionManager>>>,
) -> Result<Vec<SessionInfo>, String> {
    let manager = session_manager.lock().await;
    Ok(manager.list_active_sessions().await)
}

#[tauri::command]
pub async fn terminate_session(
    session_manager: State<'_, Arc<Mutex<SessionManager>>>,
    session_id: String,
) -> Result<(), String> {
    let manager = session_manager.lock().await;
    manager
        .terminate_session(&session_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn pause_session(
    session_manager: State<'_, Arc<Mutex<SessionManager>>>,
    session_id: String,
) -> Result<(), String> {
    let manager = session_manager.lock().await;
    manager
        .pause_session(&session_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn resume_session(
    session_manager: State<'_, Arc<Mutex<SessionManager>>>,
    session_id: String,
) -> Result<(), String> {
    let manager = session_manager.lock().await;
    manager
        .resume_session(&session_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn send_input(
    session_manager: State<'_, Arc<Mutex<SessionManager>>>,
    session_id: String,
    input: String,
) -> Result<(), String> {
    let manager = session_manager.lock().await;
    manager
        .send_input(&session_id, &input)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_multi_session_output(
    session_manager: State<'_, Arc<Mutex<SessionManager>>>,
    session_id: String,
    lines: Option<usize>,
) -> Result<Vec<String>, String> {
    let manager = session_manager.lock().await;
    let lines = lines.unwrap_or(50);
    manager
        .get_session_output(&session_id, lines)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_session_diff(
    session_manager: State<'_, Arc<Mutex<SessionManager>>>,
    session_id: String,
) -> Result<DiffStats, String> {
    let manager = session_manager.lock().await;
    manager
        .get_session_diff(&session_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_session_config(
    session_manager: State<'_, Arc<Mutex<SessionManager>>>,
    session_id: String,
    config: SessionConfig,
) -> Result<(), String> {
    let manager = session_manager.lock().await;
    manager
        .update_session_config(&session_id, config)
        .await
        .map_err(|e| e.to_string())
}

