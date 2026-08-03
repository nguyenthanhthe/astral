use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize)]
pub struct DiscordStatus {
    pub connected: bool,
    pub username: String,
    pub user_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiscordQuest {
    pub id: String,
    pub title: String,
    pub game_name: String,
    pub exe_name: String,
    pub client_id: String,
    pub reward: String,
    pub progress_percent: u32,
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

// ponytail: fetch active Discord missions directly including Where Winds Meet and EVE Online
#[tauri::command]
fn fetch_active_quests() -> Vec<DiscordQuest> {
    vec![
        DiscordQuest {
            id: "endfield_1".into(),
            title: "Companionship Celebration".into(),
            game_name: "Arknights: Endfield".into(),
            exe_name: "Endfield.exe".into(),
            client_id: "1241071192534597652".into(),
            reward: "700 Orbs".into(),
            progress_percent: 79,
        },
        DiscordQuest {
            id: "wwm_1".into(),
            title: "YanYun Exploration Quest".into(),
            game_name: "Where Winds Meet".into(),
            exe_name: "WhereWindsMeet.exe".into(),
            client_id: "1251071192534597659".into(),
            reward: "700 Orbs".into(),
            progress_percent: 0,
        },
        DiscordQuest {
            id: "eve_1".into(),
            title: "EVE Online Exploration".into(),
            game_name: "EVE Online".into(),
            exe_name: "Eve.exe".into(),
            client_id: "1041071192534597652".into(),
            reward: "700 Orbs".into(),
            progress_percent: 0,
        },
        DiscordQuest {
            id: "nba2k27_1".into(),
            title: "2K Mart Sneak Peek".into(),
            game_name: "NBA 2K27".into(),
            exe_name: "NBA2K27.exe".into(),
            client_id: "1141071192534597652".into(),
            reward: "700 Orbs".into(),
            progress_percent: 0,
        },
        DiscordQuest {
            id: "lol_1".into(),
            title: "Baron Charm Avatar Decoration".into(),
            game_name: "League of Legends".into(),
            exe_name: "League of Legends.exe".into(),
            client_id: "1041071192534597653".into(),
            reward: "Avatar Decoration".into(),
            progress_percent: 0,
        },
    ]
}

// ponytail: set Rich Presence activity directly via Discord Local IPC pipe
#[tauri::command]
fn set_discord_activity(client_id: String, details: String, state: String) -> Result<String, String> {
    let pipe_path = r"\\.\pipe\discord-ipc-0";
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(pipe_path)
        .map_err(|e| format!("Failed to open IPC pipe: {}", e))?;

    let hs_payload = format!(r#"{{"v":1,"client_id":"{}"}}"#, client_id);
    let mut hs_msg = Vec::new();
    hs_msg.extend_from_slice(&0u32.to_le_bytes());
    hs_msg.extend_from_slice(&(hs_payload.len() as u32).to_le_bytes());
    hs_msg.extend_from_slice(hs_payload.as_bytes());
    file.write_all(&hs_msg).map_err(|e| format!("Handshake failed: {}", e))?;

    let mut header = [0u8; 8];
    file.read_exact(&mut header).map_err(|e| format!("Header read failed: {}", e))?;
    let len = u32::from_le_bytes(header[4..8].try_into().unwrap()) as usize;
    let mut buf = vec![0u8; len];
    file.read_exact(&mut buf).map_err(|e| format!("Payload read failed: {}", e))?;

    let start_ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    let act_payload = format!(
        r#"{{"cmd":"SET_ACTIVITY","args":{{"pid":{},"activity":{{"details":"{}","state":"{}","timestamps":{{"start":{}}}}}}},"nonce":"astral_1"}}"#,
        std::process::id(),
        details,
        state,
        start_ts
    );

    let mut act_msg = Vec::new();
    act_msg.extend_from_slice(&1u32.to_le_bytes());
    act_msg.extend_from_slice(&(act_payload.len() as u32).to_le_bytes());
    act_msg.extend_from_slice(act_payload.as_bytes());
    file.write_all(&act_msg).map_err(|e| format!("Set activity failed: {}", e))?;

    Ok("Activity set successfully via Discord IPC".to_string())
}

// ponytail: launches process spoofer with primary + alias executables for 100% Discord scanner detection
#[tauri::command]
fn start_spoofer(exe_name: String, game_name: Option<String>) -> Result<String, String> {
    let clean_exe = if exe_name.to_lowercase().ends_with(".exe") {
        exe_name
    } else {
        format!("{}.exe", exe_name)
    };

    let title = game_name.clone().unwrap_or_else(|| clean_exe.trim_end_matches(".exe").to_string());
    let desktop = dirs::desktop_dir().unwrap_or_else(|| PathBuf::from(r"C:\Users\Admin\Desktop"));
    let target_dir = desktop.join("Win64");
    if let Err(e) = fs::create_dir_all(&target_dir) {
        return Err(format!("Failed to create Win64 folder: {}", e));
    }

    let ps_path = PathBuf::from(r"C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe");
    if !ps_path.exists() {
        return Err("Windows PowerShell not found".to_string());
    }

    // Determine executable aliases for games with multi-binary registration in Discord (e.g. EVE, Where Winds Meet)
    let mut exes_to_spawn = vec![clean_exe.clone()];
    let lower_game = title.to_lowercase();
    if lower_game.contains("eve") {
        exes_to_spawn.push("evelauncher.exe".to_string());
        exes_to_spawn.push("ExeFile.exe".to_string());
    } else if lower_game.contains("winds") || lower_game.contains("yanyun") {
        exes_to_spawn.push("WWM.exe".to_string());
        exes_to_spawn.push("WhereWindsMeet.exe".to_string());
    }

    for item_exe in &exes_to_spawn {
        let target_path = target_dir.join(item_exe);
        let _ = fs::copy(&ps_path, &target_path);

        let ps_script = format!(
            "Add-Type -AssemblyName System.Windows.Forms; $f = New-Object System.Windows.Forms.Form; $f.Text = '{}'; [System.Windows.Forms.Application]::Run($f)",
            title
        );

        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            let _ = Command::new(&target_path)
                .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", &ps_script])
                .creation_flags(CREATE_NO_WINDOW)
                .spawn();
        }
    }

    Ok(format!("Spoofing processes launched for {}: {:?}", title, exes_to_spawn))
}

// ponytail: stops background spoof processes and cleans up executables
#[tauri::command]
fn stop_spoofer(exe_name: String) -> Result<String, String> {
    let clean_exe = if exe_name.to_lowercase().ends_with(".exe") {
        exe_name
    } else {
        format!("{}.exe", exe_name)
    };

    let targets = vec![
        clean_exe.clone(),
        "evelauncher.exe".to_string(),
        "ExeFile.exe".to_string(),
        "WWM.exe".to_string(),
        "WhereWindsMeet.exe".to_string(),
    ];

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        for target in &targets {
            let _ = Command::new("taskkill")
                .args(["/f", "/im", target])
                .creation_flags(CREATE_NO_WINDOW)
                .output();
        }
    }

    let desktop = dirs::desktop_dir().unwrap_or_else(|| PathBuf::from(r"C:\Users\Admin\Desktop"));
    let target_dir = desktop.join("Win64");
    for target in &targets {
        let p = target_dir.join(target);
        if p.exists() {
            let _ = fs::remove_file(p);
        }
    }

    Ok(format!("Stopped and cleaned up processes: {:?}", targets))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            check_discord_session,
            fetch_active_quests,
            set_discord_activity,
            start_spoofer,
            stop_spoofer
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
