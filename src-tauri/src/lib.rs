use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::{Read, Write};

#[derive(Debug, Serialize, Deserialize)]
pub struct DiscordStatus {
    pub connected: bool,
    pub username: String,
    pub user_id: String,
}

// ponytail: native Windows named pipe connection to Discord IPC without external dependencies
#[tauri::command]
fn check_discord_session() -> DiscordStatus {
    let pipe_path = r"\\.\pipe\discord-ipc-0";
    if let Ok(mut file) = OpenOptions::new().read(true).write(true).open(pipe_path) {
        // Opcode 0: Handshake payload
        let payload = r#"{"v":1,"client_id":"356875221078245376"}"#;
        let mut msg = Vec::new();
        msg.extend_from_slice(&0u32.to_le_bytes());
        msg.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        msg.extend_from_slice(payload.as_bytes());

        if file.write_all(&msg).is_ok() {
            let mut header = [0u8; 8];
            if file.read_exact(&mut header).is_ok() {
                let len = u32::from_le_bytes(header[4..8].try_into().unwrap()) as usize;
                let mut buf = vec![0u8; len];
                if file.read_exact(&mut buf).is_ok() {
                    if let Ok(data) = serde_json::from_slice::<serde_json::Value>(&buf) {
                        if let Some(user) = data.get("data").and_then(|d| d.get("user")) {
                            let username = user.get("username").and_then(|u| u.as_str()).unwrap_or("telecom.no1");
                            let user_id = user.get("id").and_then(|i| i.as_str()).unwrap_or("1321316455052083264");
                            return DiscordStatus {
                                connected: true,
                                username: username.to_string(),
                                user_id: user_id.to_string(),
                            };
                        }
                    }
                }
            }
        }
    }
    DiscordStatus {
        connected: false,
        username: "Disconnected".to_string(),
        user_id: "".to_string(),
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
