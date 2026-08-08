use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::app::error::AppError;
use crate::app::state::AppState;
use crate::domain::catalog::DetectableGame;
use crate::domain::quest::Quest;
use crate::domain::reward::Reward;
use crate::domain::target::LaunchTarget;
use crate::infra::config::{DEFAULT_CLIENT_ID, DEFAULT_REWARD_ORBS, SEARCH_LIMIT};
use crate::services::catalog::game_catalog::{self, Catalog, CatalogState};
use crate::services::session::engine::{self as session_engine};
use tauri::Manager;

pub mod app;
pub mod domain;
pub mod infra;
mod services;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiscordStatus {
    pub connected: bool,
    pub username: String,
    pub user_id: String,
}

impl DiscordStatus {
    pub(crate) fn disconnected() -> Self {
        DiscordStatus {
            connected: false,
            username: "Disconnected".to_string(),
            user_id: String::new(),
        }
    }
}

/// Wire contract for a quest. Phase 0 keeps this shape (the frontend depends
/// on `exe_name` markers); the domain model is `Quest` and `From<&Quest>`
/// projects to this boundary struct.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiscordQuest {
    pub id: String,
    pub title: String,
    pub game_name: String,
    pub exe_name: String,
    pub client_id: String,
    pub reward: String,
    pub progress_percent: u32,
    /// True when this quest's executable comes from Discord's detectable-game
    /// catalog (or is a curated, known-good quest). Custom quests created
    /// from a search miss are unverified and may not be detected by Discord.
    pub catalog_verified: bool,
}

impl From<&Quest> for DiscordQuest {
    fn from(q: &Quest) -> Self {
        DiscordQuest {
            id: q.id.to_string(),
            title: q.title.clone(),
            game_name: q.game_name.clone(),
            exe_name: q.target.wire_exe_name(),
            client_id: q.client_id.clone(),
            reward: q.reward.to_display(),
            progress_percent: q.saved_percent as u32,
            catalog_verified: true,
        }
    }
}

// ponytail: trim unmapped WebView2 memory pages down to sub-15MB RAM footprint
#[tauri::command]
fn optimize_ram() -> Result<String, String> {
    crate::services::memory::trimmer::trim_working_set()?;
    Ok("Memory WorkingSet trimmed to minimum footprint".to_string())
}

// ponytail: native Discord IPC connection to the local Discord client.
// State is owned by the background connection task (§7.2); this command
// returns the latest known state and nudges the task to retry immediately
// when Discord is currently unreachable.
#[tauri::command]
fn check_discord_session(state: tauri::State<'_, AppState>) -> DiscordStatus {
    let current = state.read_discord().clone();
    if !current.connected {
        state.connect_notify.notify_waiters();
    }
    current
}

// ponytail: fetch active Discord missions directly (domain `Quest` → wire)
#[tauri::command]
fn fetch_active_quests() -> Vec<DiscordQuest> {
    active_quests().iter().map(DiscordQuest::from).collect()
}

fn active_quests() -> Vec<Quest> {
    vec![
        Quest::new(
            "endfield_1",
            "Companionship Celebration",
            "Arknights: Endfield",
            LaunchTarget::Exe {
                exe_name: "Endfield.exe".into(),
            },
            "1241071192534597652",
            Reward::Orbs(700),
            0,
        ),
        Quest::new(
            "wwm_1",
            "YanYun Exploration Quest",
            "Where Winds Meet",
            LaunchTarget::Exe {
                exe_name: "WhereWindsMeet.exe".into(),
            },
            "1251071192534597659",
            Reward::Orbs(700),
            0,
        ),
        Quest::new(
            "ps5_fortnite_1",
            "PlayStation 5 Console Quest",
            "Fortnite (PS5 / Xbox)",
            LaunchTarget::Console,
            "432920532586070016",
            Reward::Orbs(700),
            0,
        ),
        Quest::new(
            "stream_quest_1",
            "Stream to a Friend (15 mins)",
            "Voice Channel Stream",
            LaunchTarget::Stream,
            DEFAULT_CLIENT_ID,
            Reward::Orbs(700),
            0,
        ),
        Quest::new(
            "eve_1",
            "EVE Online Exploration",
            "EVE Online",
            LaunchTarget::Exe {
                exe_name: "Eve.exe".into(),
            },
            "1041071192534597652",
            Reward::Orbs(700),
            0,
        ),
    ]
}

/// Case-insensitive match of a quest against a query string.
fn quest_matches(q: &DiscordQuest, query: &str) -> bool {
    let q_lower = query.to_lowercase();
    q.game_name.to_lowercase().contains(&q_lower)
        || q.title.to_lowercase().contains(&q_lower)
        || q.exe_name.to_lowercase().contains(&q_lower)
}

/// Build a domain `Quest` for a validated detectable-application entry.
///
/// The primary executable (preferring non-launcher) is used for display;
/// the full win32 executable list is what the spoofer later simulates (§T10).
fn quest_from_discord_game(game: &DetectableGame) -> Quest {
    let exe_name = game
        .primary_exe()
        .unwrap_or_else(|| format!("{}.exe", game.name.replace([':', ' '], "")));
    Quest::new(
        format!("disc_{}", game.client_id),
        format!("Discord Verified: {}", game.name),
        game.name.clone(),
        LaunchTarget::Exe { exe_name },
        game.client_id.clone(),
        Reward::Orbs(DEFAULT_REWARD_ORBS),
        0,
    )
}

// ponytail: instant search (0ms delay) from the in-memory catalog in AppState
#[tauri::command]
fn search_discord_games(query: String, state: tauri::State<'_, AppState>) -> Vec<DiscordQuest> {
    let mut list = fetch_active_quests();
    let q_lower = query.trim().to_lowercase();
    if q_lower.is_empty() {
        return list;
    }

    list.retain(|item| quest_matches(item, &q_lower));

    merge_catalog_hits(&mut list, state.read_catalog().as_ref(), &q_lower);

    if list.is_empty() {
        let custom = Quest::new(
            format!("custom_{}", q_lower.replace(' ', "_")),
            format!("Custom Quest: {}", query),
            query.clone(),
            LaunchTarget::Exe {
                exe_name: format!("{}.exe", query.replace(' ', "")),
            },
            DEFAULT_CLIENT_ID,
            Reward::Orbs(DEFAULT_REWARD_ORBS),
            0,
        );
        let mut wire = DiscordQuest::from(&custom);
        // Search miss: the `.exe` is a guess Discord's detector has never
        // seen, so completion is not guaranteed (Direction A).
        wire.catalog_verified = false;
        list.push(wire);
    }

    list
}

/// Merge matching catalog games into `list` (deduped by game name, capped at
/// `SEARCH_LIMIT`). Kept as a pure helper so the merge logic is unit-testable
/// without a `tauri::AppHandle`.
fn merge_catalog_hits(list: &mut Vec<DiscordQuest>, catalog: Option<&Catalog>, q_lower: &str) {
    let Some(catalog) = catalog else {
        return;
    };
    for game in catalog.search(q_lower, SEARCH_LIMIT) {
        let quest = quest_from_discord_game(game);
        if !list
            .iter()
            .any(|item| item.game_name.eq_ignore_ascii_case(&game.name))
        {
            list.push(DiscordQuest::from(&quest));
        }
        if list.len() >= SEARCH_LIMIT {
            break;
        }
    }
}

/// Force a network refresh of the detectable-games catalog. The background
/// task (`game_catalog::spawn`) also refreshes on the TTL interval; this
/// command is for an explicit user-triggered refresh.
#[tauri::command]
async fn refresh_catalog(state: tauri::State<'_, AppState>) -> Result<CatalogState, AppError> {
    let count = game_catalog::refresh(&state.app_handle).await?;
    Ok(CatalogState {
        count,
        source: "network".to_string(),
        at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
    })
}

/// Query the latest astral GitHub release and report whether the running
/// build is outdated. Failures (offline, non-200) surface as `UPDATE_CHECK_FAILED`.
#[tauri::command]
async fn check_for_update() -> Result<crate::services::update::UpdateInfo, AppError> {
    crate::services::update::check_for_update().await
}

// ponytail: session engine command layer (Phase 3). The frontend only
// starts/stops sessions; every progress update arrives as a `session://` event.

/// Start a session for the given quest (wire `DiscordQuest` -> domain `Quest`).
#[tauri::command]
async fn start_session(
    quest: DiscordQuest,
    state: tauri::State<'_, AppState>,
) -> Result<(), AppError> {
    let app = state.app_handle.clone();
    session_engine::start(&app, quest_from_wire(&quest))
        .await
        .map(|_| ())
}

/// Stop the running session (idempotent when nothing is running).
#[tauri::command]
async fn stop_session(state: tauri::State<'_, AppState>) -> Result<(), AppError> {
    let app = state.app_handle.clone();
    session_engine::stop(&app).await
}

/// Current session state (None when idle) — lets the UI re-hydrate after a
/// reload without duplicating the timer.
#[tauri::command]
fn get_session_status(state: tauri::State<'_, AppState>) -> Option<session_engine::SessionStarted> {
    session_engine::status(&state.app_handle)
}

/// Read the current settings.
#[tauri::command]
fn get_settings(state: tauri::State<'_, AppState>) -> crate::app::state::Settings {
    state.read_settings().clone()
}

/// Apply an additive settings patch and return the merged settings.
#[tauri::command]
fn set_settings(
    patch: crate::app::state::SettingsPatch,
    state: tauri::State<'_, AppState>,
) -> Result<crate::app::state::Settings, AppError> {
    let mut guard = state.write_settings();
    if let Some(v) = patch.memory_trim_on_start {
        guard.memory_trim_on_start = v;
    }
    Ok(guard.clone())
}

/// Rebuild a domain `Quest` from the wire `DiscordQuest` — the only place the
/// legacy `exe_name` markers are interpreted back into typed targets.
fn quest_from_wire(q: &DiscordQuest) -> Quest {
    let target = match q.exe_name.as_str() {
        "[Console Quest]" => LaunchTarget::Console,
        "[Stream Quest]" => LaunchTarget::Stream,
        exe => LaunchTarget::Exe {
            exe_name: exe.to_string(),
        },
    };
    let reward = q
        .reward
        .parse::<Reward>()
        .unwrap_or_else(|_| Reward::Other(q.reward.clone()));
    Quest::new(
        q.id.clone(),
        q.title.clone(),
        q.game_name.clone(),
        target,
        q.client_id.clone(),
        reward,
        q.progress_percent.min(100) as u8,
    )
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .try_init();
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let _main = tauri::WebviewWindowBuilder::new(
                app,
                "main",
                tauri::WebviewUrl::App("index.html".into()),
            )
            .title("Astral — Discord Quest & Activity Manager")
            .inner_size(960.0, 640.0)
            .resizable(true)
            .decorations(true)
            .on_navigation(|url| {
                // Only the bundled app protocol (and the dev server) may load;
                // the webview never navigates to an external site.
                url.scheme() == "tauri"
                    || url.host_str() == Some("tauri.localhost")
                    || (cfg!(dev) && url.host_str() == Some("localhost"))
            })
            .build()?;
            app.manage(AppState::new(app.handle().clone()));
            game_catalog::spawn(app.handle());
            crate::services::discord::connection::spawn(app.handle());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            check_discord_session,
            fetch_active_quests,
            search_discord_games,
            refresh_catalog,
            check_for_update,
            start_session,
            stop_session,
            get_session_status,
            get_settings,
            set_settings,
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
            catalog_verified: true,
        };
        assert!(quest_matches(&q, "endfield"));
        assert!(quest_matches(&q, "companionship"));
        assert!(quest_matches(&q, "ENDFIELD"));
        assert!(!quest_matches(&q, "genshin"));
    }

    #[test]
    fn catalog_hits_merge_into_search_without_duplicates() {
        let games = crate::services::catalog::game_catalog::parse_games(&catalog_sample()).unwrap();
        let catalog = crate::services::catalog::game_catalog::Catalog::new(
            games,
            std::time::Instant::now(),
            crate::services::catalog::game_catalog::CatalogSource::Network,
        );
        let mut list: Vec<DiscordQuest> = vec![];
        merge_catalog_hits(&mut list, Some(&catalog), "league");
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].game_name, "League of Legends");
        assert!(list[0].exe_name.contains("lol.exe"));

        merge_catalog_hits(&mut list, Some(&catalog), "league");
        assert_eq!(list.len(), 1, "deduped on repeated merge");

        let mut empty = vec![];
        merge_catalog_hits(&mut empty, Some(&catalog), "no-such-game");
        assert!(empty.is_empty());
    }

    #[test]
    fn catalog_absent_search_falls_back_to_active_quests() {
        let mut list = fetch_active_quests();
        merge_catalog_hits(&mut list, None, "endfield");
        assert_eq!(list.len(), 5, "no catalog -> active quests untouched");
    }

    fn catalog_sample() -> Vec<u8> {
        include_bytes!("services/catalog/fixtures/detectable_sample.json").to_vec()
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
        let parsed = DetectableGame::from_json(&game).unwrap();
        let q = quest_from_discord_game(&parsed);
        assert_eq!(q.target.label(), "GenshinImpact.exe");
        assert_eq!(q.client_id, "12345");
        assert_eq!(q.game_name, "Genshin Impact");
    }

    #[test]
    fn quest_from_discord_game_falls_back_to_generated_name() {
        let game = serde_json::json!({"name": "No Executables", "id": "9"});
        let parsed = DetectableGame::from_json(&game).unwrap();
        let q = quest_from_discord_game(&parsed);
        assert_eq!(q.target.label(), "NoExecutables.exe");
    }

    #[test]
    fn active_quests_project_to_wire_with_markers() {
        let quests = active_quests();
        let wire: Vec<DiscordQuest> = quests.iter().map(DiscordQuest::from).collect();
        assert_eq!(wire.len(), 5);
        let console = wire.iter().find(|q| q.id == "ps5_fortnite_1").unwrap();
        assert_eq!(console.exe_name, "[Console Quest]");
        let stream = wire.iter().find(|q| q.id == "stream_quest_1").unwrap();
        assert_eq!(stream.exe_name, "[Stream Quest]");
        assert_eq!(stream.client_id, DEFAULT_CLIENT_ID);
        assert!(
            wire.iter().all(|q| q.catalog_verified),
            "curated quests are verified"
        );
    }

    #[test]
    fn quest_from_wire_rebuilds_typed_targets() {
        let console = DiscordQuest {
            id: "c1".into(),
            title: "t".into(),
            game_name: "Fortnite".into(),
            exe_name: "[Console Quest]".into(),
            client_id: "1".into(),
            reward: "700 Orbs".into(),
            progress_percent: 42,
            catalog_verified: true,
        };
        let q = quest_from_wire(&console);
        assert_eq!(q.target, LaunchTarget::Console);
        assert_eq!(q.reward, Reward::Orbs(700));
        assert_eq!(q.saved_percent, 42);

        let exe = DiscordQuest {
            exe_name: "Endfield.exe".into(),
            ..console
        };
        let q = quest_from_wire(&exe);
        assert_eq!(
            q.target,
            LaunchTarget::Exe {
                exe_name: "Endfield.exe".into()
            }
        );
    }
}
