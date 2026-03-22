use serde_json::json;
use tauri::{AppHandle, Emitter, Manager};

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

pub async fn call_claude_stream(
    app: AppHandle,
    system: String,
    user_message: String,
    stream_id: String,
) -> Result<(), String> {
    use futures::StreamExt;

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
        "stream": true,
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

    let mut byte_stream = response.bytes_stream();
    let mut line_buf = String::new();
    let mut full_text = String::new();

    while let Some(chunk_result) = byte_stream.next().await {
        let chunk = chunk_result.map_err(|e| format!("Stream read error: {}", e))?;
        line_buf.push_str(&String::from_utf8_lossy(&chunk));

        loop {
            match line_buf.find('\n') {
                None => break,
                Some(pos) => {
                    let line = line_buf[..pos].trim_end_matches('\r').to_string();
                    line_buf = line_buf[pos + 1..].to_string();

                    if let Some(data) = line.strip_prefix("data: ") {
                        if data == "[DONE]" {
                            continue;
                        }
                        if let Ok(ev) = serde_json::from_str::<serde_json::Value>(data) {
                            if ev["type"].as_str() == Some("content_block_delta") {
                                if let Some(text) = ev["delta"]["text"].as_str() {
                                    full_text.push_str(text);
                                    let _ = app.emit("chat-chunk", serde_json::json!({
                                        "stream_id": &stream_id,
                                        "text": text
                                    }));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let _ = app.emit("chat-done", serde_json::json!({
        "stream_id": &stream_id,
        "full_text": &full_text
    }));

    Ok(())
}
