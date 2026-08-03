use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct DiscordStatus {
    pub connected: bool,
    pub username: String,
    pub user_id: String,
}

#[tauri::command]
fn check_discord_session() -> DiscordStatus {
    // Basic IPC status check placeholder for Astral core
    DiscordStatus {
        connected: true,
        username: "telecom.no1".to_string(),
        user_id: "1321316455052083264".to_string(),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![check_discord_session])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
