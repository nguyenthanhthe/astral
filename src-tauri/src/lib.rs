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
    /// Real quest length in seconds, carried by mirrored quests. `None` lets
    /// the engine pick the kind-based default. Absent in older wire payloads.
    #[serde(default)]
    pub duration_secs: Option<u64>,
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
            duration_secs: None,
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

// ponytail: fetch active Discord missions directly (domain `Quest` → wire).
// Curated quests ship with a valid fallback `client_id`; the command upgrades
// each one to the app's current catalog `client_id` when the catalog is
// loaded, so the Rich Presence targets the real game application.
#[tauri::command]
fn fetch_active_quests(state: tauri::State<'_, AppState>) -> Vec<DiscordQuest> {
    fetch_active_quests_with(&state)
}

fn fetch_active_quests_with(state: &AppState) -> Vec<DiscordQuest> {
    let catalog = state.read_catalog();
    let mut list: Vec<DiscordQuest> = active_quests()
        .iter()
        .map(|q| {
            let mut wire = DiscordQuest::from(q);
            wire.client_id = resolve_quest_client_id(catalog.as_ref(), q);
            wire
        })
        .collect();
    // Quest Mirror: quests the user mirrored from their own Discord client,
    // persisted on disk (no account token involved).
    for quest in crate::services::queststore::list(&state.app_handle) {
        list.push(custom_quest_to_wire(&quest, catalog.as_ref()));
    }
    list
}

/// Project a mirrored quest to the wire. The catalog decides both the
/// `client_id` (real game application when known) and `catalog_verified`
/// (Discord's detector has actually seen this game's executable).
fn custom_quest_to_wire(
    quest: &crate::services::queststore::CustomQuest,
    catalog: Option<&Catalog>,
) -> DiscordQuest {
    let domain = quest.to_domain();
    let verified = catalog.and_then(|c| c.find(&quest.game_name)).is_some();
    DiscordQuest {
        id: domain.id.to_string(),
        title: domain.title.clone(),
        game_name: domain.game_name.clone(),
        exe_name: domain.target.wire_exe_name(),
        client_id: resolve_quest_client_id(catalog, &domain),
        reward: domain.reward.to_display(),
        progress_percent: 0,
        catalog_verified: verified,
        duration_secs: Some(quest.duration_secs),
    }
}

/// Pick the catalog `client_id` for a quest's game when the catalog is
/// available; fall back to the quest's built-in id otherwise.
fn resolve_quest_client_id(catalog: Option<&Catalog>, quest: &Quest) -> String {
    catalog
        .and_then(|c| c.find(&quest.game_name))
        .map(|g| g.client_id.clone())
        .unwrap_or_else(|| quest.client_id.clone())
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
            "1461154307171811401",
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
            "1437509662303059998",
            Reward::Orbs(700),
            0,
        ),
        Quest::new(
            "ps5_fortnite_1",
            "PlayStation 5 Console Quest",
            "Fortnite",
            LaunchTarget::Console,
            "1402418703554842694",
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
            "363413402300710912",
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
    // The game name arrives from the network catalog; sanitizing keeps the
    // generated `.exe` a plain basename (no separators, no traversal, no
    // Windows-forbidden characters) before it ever becomes a file path.
    let exe_name = game.primary_exe().unwrap_or_else(|| {
        crate::services::spoofer::orchestrator::sanitize_exe_name(&format!("{}.exe", game.name))
    });
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
    let mut list = fetch_active_quests_with(&state);
    let q_lower = query.trim().to_lowercase();
    if q_lower.is_empty() {
        return list;
    }

    list.retain(|item| quest_matches(item, &q_lower));

    merge_catalog_hits(&mut list, state.read_catalog().as_ref(), &q_lower);

    // No auto-created placeholder quest on a search miss: the old fallback
    // derived an executable path from raw search text. Explicit quest
    // creation belongs to Quest Mirror, which validates every field first.
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

// ponytail: Quest Mirror command layer — the user mirrors quests they see in
// their own Discord client; entries persist on disk and join the quest list.

/// Input shape for a mirrored quest (validated before persistence).
#[derive(Debug, Deserialize)]
pub struct CustomQuestInput {
    pub title: String,
    pub game_name: String,
    /// `"exe" | "console" | "stream" | "video"`.
    pub kind: String,
    #[serde(default)]
    pub exe_name: Option<String>,
    #[serde(default)]
    pub client_id: Option<String>,
    /// Display reward, e.g. `"700 Orbs"`.
    pub reward: String,
    pub duration_secs: u64,
}

#[tauri::command]
fn list_custom_quests(state: tauri::State<'_, AppState>) -> Vec<DiscordQuest> {
    let catalog = state.read_catalog();
    crate::services::queststore::list(&state.app_handle)
        .iter()
        .map(|quest| custom_quest_to_wire(quest, catalog.as_ref()))
        .collect()
}

#[tauri::command]
fn save_custom_quest(
    input: CustomQuestInput,
    state: tauri::State<'_, AppState>,
) -> Result<DiscordQuest, AppError> {
    let quest = crate::services::queststore::CustomQuest {
        id: crate::services::queststore::new_id(),
        title: input.title.trim().to_string(),
        game_name: input.game_name.trim().to_string(),
        kind: input.kind.trim().to_lowercase(),
        exe_name: input
            .exe_name
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()),
        client_id: input
            .client_id
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty()),
        reward: input.reward.trim().to_string(),
        duration_secs: input.duration_secs,
        created_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64,
    };
    let saved = crate::services::queststore::add(&state.app_handle, quest)?;
    let catalog = state.read_catalog();
    Ok(custom_quest_to_wire(&saved, catalog.as_ref()))
}

#[tauri::command]
fn delete_custom_quest(id: String, state: tauri::State<'_, AppState>) -> Result<(), AppError> {
    crate::services::queststore::remove(&state.app_handle, &id)?;
    Ok(())
}

// ponytail: session engine command layer (Phase 3). The frontend only
// starts/stops sessions; every progress update arrives as a `session://` event.

/// Start a session for the given quest (wire `DiscordQuest` -> domain `Quest`).
///
/// `demo = true` runs the session purely locally — the progress ring is a
/// display-only simulation with no spoofer process and no Discord traffic.
/// `duration_secs` on the quest (mirrored quests) overrides the kind-based
/// default when it is a sane value.
#[tauri::command]
async fn start_session(
    quest: DiscordQuest,
    demo: Option<bool>,
    state: tauri::State<'_, AppState>,
) -> Result<(), AppError> {
    const MIN_OVERRIDE_SEC: u64 = 10;
    const MAX_OVERRIDE_SEC: u64 = 24 * 60 * 60;
    let app = state.app_handle.clone();
    let duration_override = quest
        .duration_secs
        .filter(|secs| (MIN_OVERRIDE_SEC..=MAX_OVERRIDE_SEC).contains(secs))
        .map(std::time::Duration::from_secs);
    session_engine::start_with(
        &app,
        quest_from_wire(&quest),
        duration_override,
        demo.unwrap_or(false),
    )
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

/// Apply an additive settings patch and return the merged settings. The
/// merged value persists to `settings.json`; a write failure is logged but
/// still returns the applied value (durability, not correctness, is lost).
#[tauri::command]
fn set_settings(
    patch: crate::app::state::SettingsPatch,
    state: tauri::State<'_, AppState>,
) -> Result<crate::app::state::Settings, AppError> {
    let merged = {
        let mut guard = state.write_settings();
        if let Some(v) = patch.memory_trim_on_start {
            guard.memory_trim_on_start = v;
        }
        if let Some(v) = patch.demo_mode {
            guard.demo_mode = v;
        }
        guard.clone()
    };
    if let Err(err) = crate::infra::settings::save(&state.app_handle, &merged) {
        log::warn!("{}", err.log_detail());
    }
    Ok(merged)
}

/// Rebuild a domain `Quest` from the wire `DiscordQuest` — the only place the
/// legacy `exe_name` markers are interpreted back into typed targets.
fn quest_from_wire(q: &DiscordQuest) -> Quest {
    let target = match q.exe_name.as_str() {
        "[Console Quest]" => LaunchTarget::Console,
        "[Stream Quest]" => LaunchTarget::Stream,
        "[Video Quest]" => LaunchTarget::Video,
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
            list_custom_quests,
            save_custom_quest,
            delete_custom_quest,
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
            duration_secs: None,
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
        let quests = active_quests();
        let mut list: Vec<DiscordQuest> = quests.iter().map(DiscordQuest::from).collect();
        merge_catalog_hits(&mut list, None, "endfield");
        assert_eq!(list.len(), 5, "no catalog -> active quests untouched");
    }

    #[test]
    fn resolve_quest_client_id_uses_catalog_when_available() {
        let games = crate::services::catalog::game_catalog::parse_games(&catalog_sample()).unwrap();
        let catalog = crate::services::catalog::game_catalog::Catalog::new(
            games,
            std::time::Instant::now(),
            crate::services::catalog::game_catalog::CatalogSource::Cache,
        );
        let quests = active_quests();
        let endfield = quests.iter().find(|q| q.id == "endfield_1".into()).unwrap();
        assert_eq!(
            resolve_quest_client_id(Some(&catalog), endfield),
            "1461154307171811401"
        );

        let fallback = quests.iter().find(|q| q.id == "eve_1".into()).unwrap();
        assert_eq!(
            resolve_quest_client_id(None, fallback),
            "363413402300710912",
            "no catalog -> quest's built-in id"
        );
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
        assert_eq!(q.target.label(), "No Executables.exe");
    }

    #[test]
    fn quest_from_discord_game_fallback_cannot_traverse_paths() {
        let game = serde_json::json!({"name": "../../evil", "id": "10"});
        let parsed = DetectableGame::from_json(&game).unwrap();
        let q = quest_from_discord_game(&parsed);
        let label = q.target.label();
        assert!(!label.contains('/'), "no path separators in generated exe");
        assert!(!label.contains('\\'), "no path separators in generated exe");
        assert_eq!(label, "evil.exe");
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
            duration_secs: None,
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

    #[test]
    fn quest_from_wire_rebuilds_video_targets() {
        let video = DiscordQuest {
            id: "v1".into(),
            title: "Watch the trailer".into(),
            game_name: "YouTube".into(),
            exe_name: "[Video Quest]".into(),
            client_id: "1".into(),
            reward: "50 Orbs".into(),
            progress_percent: 0,
            catalog_verified: true,
            duration_secs: Some(30),
        };
        let q = quest_from_wire(&video);
        assert_eq!(q.target, LaunchTarget::Video);
    }

    #[test]
    fn custom_quest_wire_carries_duration_and_verification() {
        use crate::services::queststore::CustomQuest;
        let mirrored = CustomQuest {
            id: "custom_1".into(),
            title: "Watch the trailer".into(),
            game_name: "No Such Game".into(),
            kind: "video".into(),
            exe_name: None,
            client_id: None,
            reward: "50 Orbs".into(),
            duration_secs: 45,
            created_at: 0,
        };
        let wire = custom_quest_to_wire(&mirrored, None);
        assert_eq!(wire.duration_secs, Some(45));
        assert_eq!(wire.exe_name, "[Video Quest]");
        assert!(!wire.catalog_verified, "no catalog -> unverified");
        assert_eq!(wire.client_id, DEFAULT_CLIENT_ID);
    }
}
