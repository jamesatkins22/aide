mod claude;
mod commands;

use commands::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            load_tasks,
            save_tasks,
            call_claude,
            set_api_key,
            get_api_key,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
