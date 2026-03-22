use serde_json::json;
use tauri::{AppHandle, Manager};

const CLAUDE_MODEL: &str = "claude-sonnet-4-20250514";
const CLAUDE_API_URL: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_VERSION: &str = "2023-06-01";

fn config_file_path(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    Ok(data_dir.join("config.json"))
}

pub async fn call_claude(
    app: AppHandle,
    system: String,
    user_message: String,
) -> Result<String, String> {
    // Read API key from config.json
    let path = config_file_path(&app)?;
    if !path.exists() {
        return Err("No config file found. Please set your API key in Settings.".to_string());
    }
    let raw = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read config file: {}", e))?;
    let val: serde_json::Value = serde_json::from_str(&raw)
        .map_err(|e| format!("Failed to parse config file: {}", e))?;
    let api_key = val["api_key"].as_str().unwrap_or("").to_string();

    if api_key.trim().is_empty() {
        return Err("API key is empty. Please set your API key in Settings.".to_string());
    }

    let client = reqwest::Client::new();

    let body = json!({
        "model": CLAUDE_MODEL,
        "max_tokens": 1500,
        "system": system,
        "messages": [{ "role": "user", "content": user_message }]
    });

    let response = client
        .post(CLAUDE_API_URL)
        .header("x-api-key", &api_key)
        .header("anthropic-version", ANTHROPIC_VERSION)
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Network error calling Claude API: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_body = response.text().await.unwrap_or_default();
        return Err(format!("Claude API error {}: {}", status, error_body));
    }

    let response_json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse Claude API response: {}", e))?;

    let text = response_json["content"][0]["text"]
        .as_str()
        .ok_or_else(|| format!("Unexpected Claude API response: {}", response_json))?
        .to_string();

    Ok(text)
}
