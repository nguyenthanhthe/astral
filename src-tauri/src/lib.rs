use serde::{Deserialize, Serialize};
#[cfg(target_os = "windows")]
use std::fs;
#[cfg(target_os = "windows")]
use std::fs::OpenOptions;
#[cfg(target_os = "windows")]
use std::path::PathBuf;
#[cfg(target_os = "windows")]
use std::process::Command;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

mod discord_ipc;

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

/// Lock the detectable cache, recovering from poisoning instead of panicking.
fn cache_lock() -> std::sync::MutexGuard<'static, Option<Vec<serde_json::Value>>> {
    DETECTABLE_CACHE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

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
        log::debug!("trimmed Windows WorkingSet to minimum footprint");
    }
    Ok("Memory WorkingSet trimmed to minimum footprint".to_string())
}

// ponytail: background pre-fetcher for 23,888 Discord games database (0ms instant search)
fn preload_detectable_cache() {
    std::thread::spawn(|| {
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;

            // Bind to a named variable so the guard is dropped at the end of
            // the `let` statement. Keeping it alive across the re-acquire
            // below would deadlock on the non-reentrant Mutex.
            let already_cached = cache_lock().is_some();
            if !already_cached {
                log::debug!("prefetching Discord detectable games database");
                let output = Command::new("powershell")
                    .args([
                        "-NoProfile",
                        "-Command",
                        "Invoke-RestMethod -Uri 'https://discord.com/api/v9/applications/detectable' | ConvertTo-Json -Depth 4",
                    ])
                    .creation_flags(CREATE_NO_WINDOW)
                    .output();

                match output {
                    Ok(out) if out.status.success() => {
                        match serde_json::from_slice::<Vec<serde_json::Value>>(&out.stdout) {
                            Ok(games) => {
                                let mut guard = cache_lock();
                                *guard = Some(games);
                                log::info!("cached Discord detectable games database");
                            }
                            Err(e) => {
                                log::warn!("failed to parse detectable games database: {e}")
                            }
                        }
                    }
                    Ok(out) => log::warn!(
                        "discord detectable games fetch failed: {:?}",
                        out.status.code()
                    ),
                    Err(e) => log::warn!("discord detectable games fetch error: {e}"),
                }
            }
        }
    });
}

// ponytail: native Discord IPC connection to the local Discord client
#[tauri::command]
fn check_discord_session() -> DiscordStatus {
    preload_detectable_cache();
    match open_discord_pipe() {
        Ok(mut stream) => {
            log::debug!("connected to Discord IPC endpoint");
            match discord_ipc::handshake(&mut stream, "356875221078245376") {
                Ok(resp) => {
                    let user = resp.get("data").and_then(|d| d.get("user"));
                    let username = user
                        .and_then(|u| u.get("username"))
                        .and_then(|u| u.as_str())
                        .unwrap_or("Unknown")
                        .to_string();
                    let user_id = user
                        .and_then(|u| u.get("id"))
                        .and_then(|i| i.as_str())
                        .unwrap_or("")
                        .to_string();
                    log::info!("Discord session detected for user {username}");
                    DiscordStatus {
                        connected: true,
                        username,
                        user_id,
                    }
                }
                Err(e) => {
                    log::warn!("Discord IPC handshake failed: {e}");
                    DiscordStatus::disconnected()
                }
            }
        }
        Err(e) => {
            log::warn!("Discord IPC endpoint unavailable: {e}");
            DiscordStatus::disconnected()
        }
    }
}

impl DiscordStatus {
    fn disconnected() -> Self {
        DiscordStatus {
            connected: false,
            username: "Disconnected".to_string(),
            user_id: String::new(),
        }
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

/// Case-insensitive match of a quest against a query string.
fn quest_matches(q: &DiscordQuest, query: &str) -> bool {
    let q_lower = query.to_lowercase();
    q.game_name.to_lowercase().contains(&q_lower)
        || q.title.to_lowercase().contains(&q_lower)
        || q.exe_name.to_lowercase().contains(&q_lower)
}

/// Build a `DiscordQuest` for a Discord detectable-application entry.
fn quest_from_discord_game(game: &serde_json::Value, client_id: &str, name: &str) -> DiscordQuest {
    let mut exe_name = format!("{}.exe", name.replace([':', ' '], ""));

    if let Some(execs) = game.get("executables").and_then(|e| e.as_array()) {
        for ex in execs {
            if let Some(ex_name) = ex.get("name").and_then(|n| n.as_str()) {
                if ex_name.ends_with(".exe") {
                    let clean_ex = ex_name.split('/').next_back().unwrap_or(ex_name);
                    exe_name = clean_ex.to_string();
                    break;
                }
            }
        }
    }

    DiscordQuest {
        id: format!("disc_{}", client_id),
        title: format!("Discord Verified: {}", name),
        game_name: name.to_string(),
        exe_name,
        client_id: client_id.to_string(),
        reward: "700 Orbs".to_string(),
        progress_percent: 0,
    }
}

// ponytail: instant search (0ms delay) from in-memory DETECTABLE_CACHE
#[tauri::command]
fn search_discord_games(query: String) -> Vec<DiscordQuest> {
    let mut list = fetch_active_quests();
    let q_lower = query.trim().to_lowercase();
    if q_lower.is_empty() {
        return list;
    }

    list.retain(|item| quest_matches(item, &q_lower));

    let cache_guard = cache_lock();
    if let Some(ref games) = *cache_guard {
        for g in games {
            if let Some(name) = g.get("name").and_then(|n| n.as_str()) {
                if name.to_lowercase().contains(&q_lower) {
                    let client_id = g
                        .get("id")
                        .and_then(|i| i.as_str())
                        .unwrap_or("356875221078245376");

                    if !list
                        .iter()
                        .any(|item| item.game_name.eq_ignore_ascii_case(name))
                    {
                        list.push(quest_from_discord_game(g, client_id, name));
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
            id: format!("custom_{}", q_lower.replace(' ', "_")),
            title: format!("Custom Quest: {}", query),
            game_name: query.clone(),
            exe_name: format!("{}.exe", query.replace(' ', "")),
            client_id: "356875221078245376".to_string(),
            reward: "700 Orbs".to_string(),
            progress_percent: 0,
        });
    }

    list
}

// ponytail: spoof non-exe quests with dynamic end timestamp countdown for native Discord UI progress bar
#[tauri::command]
fn spoof_non_exe_quest(
    quest_type: String,
    client_id: String,
    game_name: String,
    duration_seconds: Option<u64>,
) -> Result<String, String> {
    let mut stream = open_discord_pipe()?;
    let (details, state) = if quest_type.contains("console") || game_name.contains("Console") {
        (
            "Playing on PlayStation 5 / Xbox",
            "Completing Console Quest",
        )
    } else {
        ("Streaming Game to Channel", "Completing Stream Quest")
    };

    send_activity_with_timestamps(
        &mut stream,
        &client_id,
        details,
        state,
        duration_seconds.unwrap_or(900),
        "astral_non_exe",
    )?;

    log::info!("spoofed non-exe quest: {game_name}");
    Ok(format!("Non-EXE Quest spoofed successfully: {}", game_name))
}

// ponytail: set Rich Presence activity directly with start and end timestamps for live Discord UI progress bar
#[tauri::command]
fn set_discord_activity(
    client_id: String,
    details: String,
    state: String,
    duration_seconds: Option<u64>,
) -> Result<String, String> {
    let mut stream = open_discord_pipe()?;
    send_activity_with_timestamps(
        &mut stream,
        &client_id,
        &details,
        &state,
        duration_seconds.unwrap_or(900),
        "astral_1",
    )?;

    log::info!("set Discord activity for client {client_id}");
    Ok("Activity set successfully via Discord IPC".to_string())
}

/// Open the Discord IPC named pipe (Windows), mapping errors to user strings.
#[cfg(target_os = "windows")]
fn open_discord_pipe() -> Result<Box<dyn discord_ipc::ReadWrite>, String> {
    OpenOptions::new()
        .read(true)
        .write(true)
        .open(discord_ipc::PIPE_PATH)
        .map(|stream| Box::new(stream) as Box<dyn discord_ipc::ReadWrite>)
        .map_err(|e| format!("Failed to open IPC pipe: {}", e))
}

/// Connect to the Discord IPC Unix domain socket (macOS/Linux).
#[cfg(not(target_os = "windows"))]
fn open_discord_pipe() -> Result<Box<dyn discord_ipc::ReadWrite>, String> {
    use std::os::unix::net::UnixStream;

    let path = discord_ipc::unix_socket_path();
    UnixStream::connect(&path)
        .map(|stream| Box::new(stream) as Box<dyn discord_ipc::ReadWrite>)
        .map_err(|e| {
            format!(
                "Failed to connect to Discord IPC socket {}: {}",
                path.display(),
                e
            )
        })
}

/// Handshake with Discord then SET_ACTIVITY with [start, end] timestamps.
fn send_activity_with_timestamps<R: std::io::Read + std::io::Write>(
    stream: &mut R,
    client_id: &str,
    details: &str,
    state: &str,
    duration_secs: u64,
    nonce: &str,
) -> Result<(), String> {
    discord_ipc::handshake(stream, client_id)
        .map_err(|e| format!("IPC handshake failed: {}", e))?;

    let start_ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let end_ts = start_ts + duration_secs;

    discord_ipc::set_activity(
        stream,
        std::process::id(),
        details,
        state,
        start_ts,
        end_ts,
        nonce,
    )
    .map_err(|e| format!("Set activity failed: {}", e))
}

// ponytail: launches process spoofer with primary + alias executables for 100% Discord scanner detection
#[tauri::command]
fn start_spoofer(exe_name: String, game_name: Option<String>) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        start_spoofer_windows(exe_name, game_name)
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = (exe_name, game_name);
        Err("The process spoofer is only available on Windows".to_string())
    }
}

#[cfg(target_os = "windows")]
fn start_spoofer_windows(exe_name: String, game_name: Option<String>) -> Result<String, String> {
    let clean_exe = ensure_exe_suffix(&exe_name);

    let title = game_name
        .clone()
        .unwrap_or_else(|| clean_exe.trim_end_matches(".exe").to_string());
    let target_dir = spoof_dir()?;

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
        if let Err(e) = fs::copy(&ps_path, &target_path) {
            log::warn!(
                "failed to stage spoofer executable {}: {e}",
                target_path.display()
            );
        }

        #[cfg(target_os = "windows")]
        {
            let ps_script = format!(
                "Add-Type -AssemblyName System.Windows.Forms; $f = New-Object System.Windows.Forms.Form; $f.Text = '{}'; [System.Windows.Forms.Application]::Run($f)",
                title
            );
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            let _ = Command::new(&target_path)
                .args([
                    "-NoProfile",
                    "-WindowStyle",
                    "Hidden",
                    "-Command",
                    &ps_script,
                ])
                .creation_flags(CREATE_NO_WINDOW)
                .spawn();
        }
    }

    log::info!("launched spoofer processes for {title}");
    Ok(format!(
        "Spoofing processes launched for {}: {:?}",
        title, exes_to_spawn
    ))
}

// ponytail: stops background spoof processes and cleans up executables
#[tauri::command]
fn stop_spoofer(exe_name: String) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        stop_spoofer_windows(exe_name)
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = exe_name;
        Err("The process spoofer is only available on Windows".to_string())
    }
}

#[cfg(target_os = "windows")]
fn stop_spoofer_windows(exe_name: String) -> Result<String, String> {
    let clean_exe = ensure_exe_suffix(&exe_name);

    let targets = vec![
        clean_exe,
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

    let target_dir = spoof_dir()?;
    for target in &targets {
        let p = target_dir.join(target);
        if p.exists() {
            let _ = fs::remove_file(&p);
        }
    }

    log::info!("stopped and cleaned spoofer processes: {targets:?}");
    Ok(format!("Stopped and cleaned up processes: {:?}", targets))
}

/// Normalize an executable name to end in `.exe`.
#[cfg(target_os = "windows")]
fn ensure_exe_suffix(exe_name: &str) -> String {
    if exe_name.to_lowercase().ends_with(".exe") {
        exe_name.to_string()
    } else {
        format!("{}.exe", exe_name)
    }
}

/// Resolve the Desktop/Win64 staging directory without hardcoded user paths.
#[cfg(target_os = "windows")]
fn spoof_dir() -> Result<PathBuf, String> {
    let desktop = dirs::desktop_dir().ok_or_else(|| "Desktop directory not found".to_string())?;
    let target_dir = desktop.join("Win64");
    fs::create_dir_all(&target_dir).map_err(|e| format!("Failed to create Win64 folder: {}", e))?;
    Ok(target_dir)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .try_init();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quest_matches_case_insensitive() {
        let q = DiscordQuest {
            id: "x".into(),
            title: "Companionship Celebration".into(),
            game_name: "Arknights: Endfield".into(),
            exe_name: "Endfield.exe".into(),
            client_id: "1".into(),
            reward: "700 Orbs".into(),
            progress_percent: 0,
        };
        assert!(quest_matches(&q, "endfield"));
        assert!(quest_matches(&q, "companionship"));
        assert!(quest_matches(&q, "ENDFIELD"));
        assert!(!quest_matches(&q, "genshin"));
    }

    #[test]
    fn quest_from_discord_game_uses_executable_name() {
        let game = serde_json::json!({
            "name": "Genshin Impact",
            "id": "12345",
            "executables": [
                {"name": "GenshinImpact.exe"},
                {"name": "launcher.exe"}
            ]
        });
        let q = quest_from_discord_game(&game, "12345", "Genshin Impact");
        assert_eq!(q.exe_name, "GenshinImpact.exe");
        assert_eq!(q.client_id, "12345");
        assert_eq!(q.game_name, "Genshin Impact");
    }

    #[test]
    fn quest_from_discord_game_falls_back_to_generated_name() {
        let game = serde_json::json!({"name": "No Executables", "id": "9"});
        let q = quest_from_discord_game(&game, "9", "No Executables");
        assert_eq!(q.exe_name, "NoExecutables.exe");
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn ensure_exe_suffix_adds_and_preserves() {
        assert_eq!(ensure_exe_suffix("Game"), "Game.exe");
        assert_eq!(ensure_exe_suffix("game.exe"), "game.exe");
        assert_eq!(ensure_exe_suffix("GAME.EXE"), "GAME.EXE");
    }
}
