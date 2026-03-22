use std::path::PathBuf;
use tauri::{AppHandle, Manager};

fn tasks_file_path(app: &AppHandle) -> Result<PathBuf, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    Ok(data_dir.join("tasks.json"))
}

fn config_file_path(app: &AppHandle) -> Result<PathBuf, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    Ok(data_dir.join("config.json"))
}

fn ensure_dir(app: &AppHandle) -> Result<(), String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    std::fs::create_dir_all(&data_dir)
        .map_err(|e| format!("Failed to create app data directory: {}", e))
}

#[tauri::command]
pub async fn load_tasks(app: AppHandle) -> Result<String, String> {
    let path = tasks_file_path(&app)?;
    if !path.exists() {
        return Ok(r#"{"tasks":[],"projects":{},"lastUpdated":null}"#.to_string());
    }
    std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read tasks file: {}", e))
}

#[tauri::command]
pub async fn save_tasks(app: AppHandle, state: String) -> Result<(), String> {
    ensure_dir(&app)?;
    let path = tasks_file_path(&app)?;
    let tmp_path = path.with_extension("json.tmp");
    std::fs::write(&tmp_path, &state)
        .map_err(|e| format!("Failed to write temporary file: {}", e))?;
    std::fs::rename(&tmp_path, &path)
        .map_err(|e| format!("Failed to rename temporary file: {}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn set_api_key(app: AppHandle, key: String) -> Result<(), String> {
    ensure_dir(&app)?;
    let path = config_file_path(&app)?;
    let json = serde_json::json!({ "api_key": key });
    std::fs::write(&path, json.to_string())
        .map_err(|e| format!("Failed to write config file: {}", e))
}

#[tauri::command]
pub async fn get_api_key(app: AppHandle) -> Result<String, String> {
    let path = config_file_path(&app)?;
    if !path.exists() {
        return Ok(String::new());
    }
    let raw = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read config file: {}", e))?;
    let val: serde_json::Value = serde_json::from_str(&raw)
        .map_err(|e| format!("Failed to parse config file: {}", e))?;
    Ok(val["api_key"].as_str().unwrap_or("").to_string())
}

#[tauri::command]
pub async fn call_claude(
    app: AppHandle,
    system: String,
    user_message: String,
) -> Result<String, String> {
    crate::claude::call_claude(app, system, user_message).await
}

#[tauri::command]
pub async fn call_claude_stream(
    app: AppHandle,
    system: String,
    user_message: String,
    stream_id: String,
) -> Result<(), String> {
    crate::claude::call_claude_stream(app, system, user_message, stream_id).await
}
