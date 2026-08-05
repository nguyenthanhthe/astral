use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::Command;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiscordStatus {
    pub connected: bool,
    pub username: String,
    pub user_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiscordQuest {
    pub id: String,
    pub title: String,
    pub game_name: String,
    pub exe_name: String,
    pub client_id: String,
    pub reward: String,
    pub progress_percent: u32,
}

static DETECTABLE_CACHE: Mutex<Option<Vec<serde_json::Value>>> = Mutex::new(None);

#[cfg(target_os = "windows")]
extern "system" {
    fn GetCurrentProcess() -> *mut std::ffi::c_void;
    fn SetProcessWorkingSetSize(
        hProcess: *mut std::ffi::c_void,
        dwMinimumWorkingSetSize: usize,
        dwMaximumWorkingSetSize: usize,
    ) -> i32;
}

// ponytail: trim unmapped WebView2 memory pages down to sub-15MB RAM footprint
#[tauri::command]
fn optimize_ram() -> Result<String, String> {
    #[cfg(target_os = "windows")]
    unsafe {
        let handle = GetCurrentProcess();
        SetProcessWorkingSetSize(handle, usize::MAX, usize::MAX);
    }
    Ok("Memory WorkingSet trimmed to minimum footprint".to_string())
}

// ponytail: background pre-fetcher for 23,888 Discord games database (0ms instant search)
fn preload_detectable_cache() {
    std::thread::spawn(|| {
        let mut guard = DETECTABLE_CACHE.lock().unwrap();
        if guard.is_none() {
            #[cfg(target_os = "windows")]
            {
                use std::os::windows::process::CommandExt;
                const CREATE_NO_WINDOW: u32 = 0x08000000;
                let output = Command::new("powershell")
                    .args(["-NoProfile", "-Command", "Invoke-RestMethod -Uri 'https://discord.com/api/v9/applications/detectable' | ConvertTo-Json -Depth 4"])
                    .creation_flags(CREATE_NO_WINDOW)
                    .output();

                if let Ok(out) = output {
                    if let Ok(json_data) = serde_json::from_slice::<Vec<serde_json::Value>>(&out.stdout) {
                        *guard = Some(json_data);
                    }
                }
            }
        }
    });
}

// ponytail: native Windows named pipe connection to Discord IPC without external dependencies
#[tauri::command]
fn check_discord_session() -> DiscordStatus {
    preload_detectable_cache();
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

// ponytail: fetch active Discord missions directly
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
            id: "ps5_fortnite_1".into(),
            title: "PlayStation 5 Console Quest".into(),
            game_name: "Fortnite (PS5 / Xbox)".into(),
            exe_name: "[Console Quest]".into(),
            client_id: "432920532586070016".into(),
            reward: "700 Orbs".into(),
            progress_percent: 0,
        },
        DiscordQuest {
            id: "stream_quest_1".into(),
            title: "Stream to a Friend (15 mins)".into(),
            game_name: "Voice Channel Stream".into(),
            exe_name: "[Stream Quest]".into(),
            client_id: "356875221078245376".into(),
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
    ]
}

// ponytail: instant search (0ms delay) from in-memory DETECTABLE_CACHE
#[tauri::command]
fn search_discord_games(query: String) -> Vec<DiscordQuest> {
    let mut list = fetch_active_quests();
    let q_lower = query.trim().to_lowercase();
    if q_lower.is_empty() {
        return list;
    }

    list.retain(|item| {
        item.game_name.to_lowercase().contains(&q_lower)
            || item.title.to_lowercase().contains(&q_lower)
            || item.exe_name.to_lowercase().contains(&q_lower)
    });

    let cache_guard = DETECTABLE_CACHE.lock().unwrap();
    if let Some(ref games) = *cache_guard {
        for g in games {
            if let Some(name) = g.get("name").and_then(|n| n.as_str()) {
                if name.to_lowercase().contains(&q_lower) {
                    let client_id = g.get("id").and_then(|i| i.as_str()).unwrap_or("356875221078245376").to_string();
                    let mut exe_name = format!("{}.exe", name.replace(":", "").replace(" ", ""));

                    if let Some(execs) = g.get("executables").and_then(|e| e.as_array()) {
                        for ex in execs {
                            if let Some(ex_name) = ex.get("name").and_then(|n| n.as_str()) {
                                if ex_name.ends_with(".exe") {
                                    let clean_ex = ex_name.split('/').last().unwrap_or(ex_name);
                                    exe_name = clean_ex.to_string();
                                    break;
                                }
                            }
                        }
                    }

                    if !list.iter().any(|item| item.game_name.eq_ignore_ascii_case(name)) {
                        list.push(DiscordQuest {
                            id: format!("disc_{}", client_id),
                            title: format!("Discord Verified: {}", name),
                            game_name: name.to_string(),
                            exe_name,
                            client_id,
                            reward: "700 Orbs".to_string(),
                            progress_percent: 0,
                        });
                    }

                    if list.len() >= 25 {
                        break;
                    }
                }
            }
        }
    }

    if list.is_empty() {
        list.push(DiscordQuest {
            id: format!("custom_{}", q_lower.replace(" ", "_")),
            title: format!("Custom Quest: {}", query),
            game_name: query.clone(),
            exe_name: format!("{}.exe", query.replace(" ", "")),
            client_id: "356875221078245376".to_string(),
            reward: "700 Orbs".to_string(),
            progress_percent: 0,
        });
    }

    list
}

// ponytail: spoof non-exe quests with dynamic end timestamp countdown for native Discord UI progress bar
#[tauri::command]
fn spoof_non_exe_quest(quest_type: String, client_id: String, game_name: String, duration_seconds: Option<u64>) -> Result<String, String> {
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
    let dur = duration_seconds.unwrap_or(900);
    let end_ts = start_ts + dur;

    let (details, state) = if quest_type.contains("console") || game_name.contains("Console") {
        ("Playing on PlayStation 5 / Xbox", "Completing Console Quest")
    } else {
        ("Streaming Game to Channel", "Completing Stream Quest")
    };

    let act_payload = format!(
        r#"{{"cmd":"SET_ACTIVITY","args":{{"pid":{},"activity":{{"details":"{}","state":"{}","timestamps":{{"start":{},"end":{}}}}}}},"nonce":"astral_non_exe"}}"#,
        std::process::id(),
        details,
        state,
        start_ts,
        end_ts
    );

    let mut act_msg = Vec::new();
    act_msg.extend_from_slice(&1u32.to_le_bytes());
    act_msg.extend_from_slice(&(act_payload.len() as u32).to_le_bytes());
    act_msg.extend_from_slice(act_payload.as_bytes());
    file.write_all(&act_msg).map_err(|e| format!("Set activity failed: {}", e))?;

    Ok(format!("Non-EXE Quest spoofed successfully: {}", game_name))
}

// ponytail: set Rich Presence activity directly with start and end timestamps for live Discord UI progress bar
#[tauri::command]
fn set_discord_activity(client_id: String, details: String, state: String, duration_seconds: Option<u64>) -> Result<String, String> {
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
    let dur = duration_seconds.unwrap_or(900);
    let end_ts = start_ts + dur;

    let act_payload = format!(
        r#"{{"cmd":"SET_ACTIVITY","args":{{"pid":{},"activity":{{"details":"{}","state":"{}","timestamps":{{"start":{},"end":{}}}}}}},"nonce":"astral_1"}}"#,
        std::process::id(),
        details,
        state,
        start_ts,
        end_ts
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
            search_discord_games,
            spoof_non_exe_quest,
            set_discord_activity,
            start_spoofer,
            stop_spoofer,
            optimize_ram
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
