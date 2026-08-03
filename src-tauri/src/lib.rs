use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::Command;

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

// ponytail: process spoofer creates named dummy exe and launches background process for Discord scanner detection
#[tauri::command]
fn start_spoofer(exe_name: String) -> Result<String, String> {
    let clean_exe = if exe_name.to_lowercase().ends_with(".exe") {
        exe_name
    } else {
        format!("{}.exe", exe_name)
    };

    let desktop = dirs::desktop_dir().unwrap_or_else(|| PathBuf::from(r"C:\Users\Admin\Desktop"));
    let target_dir = desktop.join("Win64");
    if let Err(e) = fs::create_dir_all(&target_dir) {
        return Err(format!("Failed to create Win64 folder: {}", e));
    }

    let target_path = target_dir.join(&clean_exe);
    let system_cmd = PathBuf::from(r"C:\Windows\System32\cmd.exe");
    if !system_cmd.exists() {
        return Err("System cmd.exe not found".to_string());
    }

    if let Err(e) = fs::copy(&system_cmd, &target_path) {
        return Err(format!("Failed to copy binary to {}: {}", target_path.display(), e));
    }

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        let _ = Command::new(&target_path)
            .args(["/c", "ping 127.0.0.1 -n 900 > nul"])
            .creation_flags(CREATE_NO_WINDOW)
            .spawn();
    }

    Ok(format!("Spoofing process launched: {}", clean_exe))
}

// ponytail: stops background spoof process and cleans up executable
#[tauri::command]
fn stop_spoofer(exe_name: String) -> Result<String, String> {
    let clean_exe = if exe_name.to_lowercase().ends_with(".exe") {
        exe_name
    } else {
        format!("{}.exe", exe_name)
    };

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        let _ = Command::new("taskkill")
            .args(["/f", "/im", &clean_exe])
            .creation_flags(CREATE_NO_WINDOW)
            .output();
    }

    let desktop = dirs::desktop_dir().unwrap_or_else(|| PathBuf::from(r"C:\Users\Admin\Desktop"));
    let target_path = desktop.join("Win64").join(&clean_exe);
    if target_path.exists() {
        let _ = fs::remove_file(target_path);
    }

    Ok(format!("Stopped and cleaned up: {}", clean_exe))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            check_discord_session,
            start_spoofer,
            stop_spoofer
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
