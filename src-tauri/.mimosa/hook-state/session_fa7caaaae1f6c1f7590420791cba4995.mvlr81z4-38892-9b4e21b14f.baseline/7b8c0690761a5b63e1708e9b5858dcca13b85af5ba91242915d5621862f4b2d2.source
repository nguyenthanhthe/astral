//! Quest Mirror — on-disk store for quests the user mirrors from their own
//! Discord client.
//!
//! Astral never touches the Discord account token, so the real active-quest
//! list cannot be read from the REST API. Instead the user mirrors each quest
//! they see in the client once; entries persist in `custom_quests.json` under
//! the platform data dir and are merged into the quest list next to the
//! curated presets. Everything here is pure disk I/O — no Discord traffic.

use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use crate::app::error::AppError;
use crate::domain::quest::Quest;
use crate::domain::reward::Reward;
use crate::domain::target::LaunchTarget;
use crate::infra::config::DEFAULT_CLIENT_ID;

pub const STORE_FILE: &str = "custom_quests.json";

/// Mirrored quest session lengths must stay in this window (10s .. 24h).
pub const MIN_DURATION_SEC: u64 = 10;
pub const MAX_DURATION_SEC: u64 = 24 * 60 * 60;
/// Orb rewards are clamped to a sane upper bound.
pub const MAX_ORBS: u32 = 100_000;

const MAX_TITLE: usize = 120;
const MAX_GAME_NAME: usize = 80;
const MAX_EXE_NAME: usize = 120;
const MAX_REWARD_LABEL: usize = 40;
const MAX_CLIENT_ID: usize = 20;

/// A quest the user mirrored from their Discord client. Unlike the curated
/// presets this carries the real quest length (`duration_secs`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CustomQuest {
    pub id: String,
    pub title: String,
    pub game_name: String,
    /// `"exe" | "console" | "stream" | "video"`.
    pub kind: String,
    /// Required for `exe` quests; empty for the IPC-only kinds.
    #[serde(default)]
    pub exe_name: Option<String>,
    /// Discord application id, when the user knows it; resolved from the
    /// catalog at merge time otherwise.
    #[serde(default)]
    pub client_id: Option<String>,
    /// Display reward, e.g. `"700 Orbs"`.
    pub reward: String,
    pub duration_secs: u64,
    #[serde(default)]
    pub created_at: u64,
}

impl CustomQuest {
    /// Validate user input before it reaches the store. Messages are
    /// user-safe (`AppError::Validation` surfaces verbatim in the UI).
    pub fn validate(&self) -> Result<(), AppError> {
        let title = self.title.trim();
        if title.is_empty() || title.chars().count() > MAX_TITLE {
            return Err(AppError::Validation(
                "Quest title must be 1–120 characters.".into(),
            ));
        }
        let game = self.game_name.trim();
        if game.is_empty() || game.chars().count() > MAX_GAME_NAME {
            return Err(AppError::Validation(
                "Game name must be 1–80 characters.".into(),
            ));
        }
        match self.kind.as_str() {
            "exe" | "console" | "stream" | "video" => {}
            _ => {
                return Err(AppError::Validation(
                    "Quest type must be exe, console, stream or video.".into(),
                ))
            }
        }
        if self.kind == "exe" {
            validate_exe_name(
                self.exe_name.as_deref().unwrap_or_default(),
            )
            .map_err(AppError::Validation)?;
        }
        if !(MIN_DURATION_SEC..=MAX_DURATION_SEC).contains(&self.duration_secs) {
            return Err(AppError::Validation(format!(
                "Duration must be {MIN_DURATION_SEC}–{MAX_DURATION_SEC} seconds."
            )));
        }
        match self.reward.parse::<Reward>() {
            Ok(Reward::Orbs(n)) if (1..=MAX_ORBS).contains(&n) => {}
            Ok(Reward::Orbs(_)) => {
                return Err(AppError::Validation(format!(
                    "Orb reward must be 1–{MAX_ORBS}."
                )))
            }
            Ok(Reward::Other(label)) => {
                let label = label.trim();
                if label.is_empty() || label.chars().count() > MAX_REWARD_LABEL {
                    return Err(AppError::Validation(
                        "Reward must be an orb amount or a label of 1–40 characters.".into(),
                    ));
                }
            }
            Err(()) => {
                return Err(AppError::Validation(
                    "Reward must be an orb amount or a short label.".into(),
                ))
            }
        }
        if let Some(id) = &self.client_id {
            let id = id.trim();
            if !id.is_empty()
                && (id.len() > MAX_CLIENT_ID || !id.chars().all(|c| c.is_ascii_digit()))
            {
                return Err(AppError::Validation(
                    "Client id must be numeric (Discord application id).".into(),
                ));
            }
        }
        Ok(())
    }

    /// Project to the domain model. The wire projection (with catalog
    /// resolution) lives in `lib.rs`, which owns the boundary.
    pub fn to_domain(&self) -> Quest {
        let target = match self.kind.as_str() {
            "console" => LaunchTarget::Console,
            "stream" => LaunchTarget::Stream,
            "video" => LaunchTarget::Video,
            _ => LaunchTarget::Exe {
                exe_name: self.exe_name.clone().unwrap_or_else(|| {
                    format!("{}.exe", self.game_name.replace([':', ' '], ""))
                }),
            },
        };
        let reward = self
            .reward
            .parse::<Reward>()
            .unwrap_or_else(|_| Reward::Other(self.reward.clone()));
        Quest::new(
            self.id.clone(),
            self.title.clone(),
            self.game_name.clone(),
            target,
            self.client_id
                .as_deref()
                .filter(|id| !id.trim().is_empty())
                .unwrap_or(DEFAULT_CLIENT_ID),
            reward,
            0,
        )
    }
}

/// Executable names are staged/looked up by file name only — never a path.
fn validate_exe_name(name: &str) -> Result<(), String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("Executable name is required for exe quests.".into());
    }
    if name.chars().count() > MAX_EXE_NAME {
        return Err("Executable name must be at most 120 characters.".into());
    }
    if !name.to_lowercase().ends_with(".exe") {
        return Err("Executable name must end with .exe.".into());
    }
    let forbidden = ['/', '\\', ':', '*', '?', '"', '<', '>', '|'];
    if name.chars().any(|c| forbidden.contains(&c) || c.is_control()) {
        return Err("Executable name must be a plain file name (no paths).".into());
    }
    Ok(())
}

/// Serialize writes so concurrent save/delete calls from the webview can't
/// interleave read-modify-write cycles and drop entries.
fn write_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

/// Load the store from a directory. A missing or corrupt file reads as empty;
/// the next successful save rewrites it from scratch.
pub fn load_from(dir: &Path) -> Vec<CustomQuest> {
    let bytes = match std::fs::read(dir.join(STORE_FILE)) {
        Ok(bytes) => bytes,
        Err(_) => return Vec::new(),
    };
    serde_json::from_slice(&bytes).unwrap_or_default()
}

fn persist_to(dir: &Path, quests: &[CustomQuest]) -> Result<(), AppError> {
    std::fs::create_dir_all(dir)
        .map_err(|e| AppError::Internal(format!("quest store dir: {e}")))?;
    let bytes = serde_json::to_vec_pretty(quests)
        .map_err(|e| AppError::Internal(format!("quest store encode: {e}")))?;
    std::fs::write(dir.join(STORE_FILE), bytes)
        .map_err(|e| AppError::Internal(format!("quest store write: {e}")))
}

fn dir_for(app: &AppHandle) -> Result<PathBuf, AppError> {
    app.path()
        .app_data_dir()
        .map_err(|e| AppError::Internal(format!("app data dir: {e}")))
}

/// All mirrored quests, oldest first.
pub fn list(app: &AppHandle) -> Vec<CustomQuest> {
    dir_for(app).map(|dir| load_from(&dir)).unwrap_or_default()
}

/// Validate and append a quest. The id must be unique (generated by
/// [`new_id`]).
pub fn add(app: &AppHandle, quest: CustomQuest) -> Result<CustomQuest, AppError> {
    let dir = dir_for(app)?;
    let _guard = write_lock()
        .lock()
        .map_err(|e| AppError::Internal(format!("quest store lock: {e}")))?;
    add_to(&dir, quest)
}

fn add_to(dir: &Path, quest: CustomQuest) -> Result<CustomQuest, AppError> {
    quest.validate()?;
    let mut quests = load_from(dir);
    if quests.iter().any(|q| q.id == quest.id) {
        return Err(AppError::Validation(
            "A quest with this id already exists.".into(),
        ));
    }
    quests.push(quest.clone());
    persist_to(dir, &quests)?;
    Ok(quest)
}

/// Remove a mirrored quest by id. Returns whether anything was removed.
pub fn remove(app: &AppHandle, id: &str) -> Result<bool, AppError> {
    let dir = dir_for(app)?;
    let _guard = write_lock()
        .lock()
        .map_err(|e| AppError::Internal(format!("quest store lock: {e}")))?;
    remove_from(&dir, id)
}

fn remove_from(dir: &Path, id: &str) -> Result<bool, AppError> {
    let mut quests = load_from(dir);
    let before = quests.len();
    quests.retain(|q| q.id != id);
    let removed = quests.len() != before;
    if removed {
        persist_to(dir, &quests)?;
    }
    Ok(removed)
}

/// Fresh unique id for a new mirror, e.g. `custom_1730000000123`.
pub fn new_id() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("custom_{millis}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> CustomQuest {
        CustomQuest {
            id: "custom_1".into(),
            title: "Watch the trailer".into(),
            game_name: "Some Game".into(),
            kind: "video".into(),
            exe_name: None,
            client_id: None,
            reward: "50 Orbs".into(),
            duration_secs: 30,
            created_at: 0,
        }
    }

    #[test]
    fn valid_quest_passes() {
        assert!(sample().validate().is_ok());
        let exe = CustomQuest {
            kind: "exe".into(),
            exe_name: Some("Endfield.exe".into()),
            reward: "700 Orbs".into(),
            duration_secs: 900,
            ..sample()
        };
        assert!(exe.validate().is_ok());
    }

    #[test]
    fn rejects_bad_kind_and_duration() {
        assert!(CustomQuest {
            kind: "rpc".into(),
            ..sample()
        }
        .validate()
        .is_err());
        assert!(CustomQuest {
            duration_secs: 3,
            ..sample()
        }
        .validate()
        .is_err());
        assert!(CustomQuest {
            duration_secs: MAX_DURATION_SEC + 1,
            ..sample()
        }
        .validate()
        .is_err());
    }

    #[test]
    fn rejects_exe_names_that_are_paths() {
        for bad in ["..\\evil.exe", "C:/evil.exe", "game"] {
            let quest = CustomQuest {
                kind: "exe".into(),
                exe_name: Some(bad.to_string()),
                ..sample()
            };
            assert!(quest.validate().is_err(), "should reject {bad:?}");
        }
        // A plain name with spaces survives.
        let ok = CustomQuest {
            kind: "exe".into(),
            exe_name: Some("Where Winds Meet.exe".into()),
            ..sample()
        };
        assert!(ok.validate().is_ok());
    }

    #[test]
    fn rejects_out_of_range_rewards() {
        assert!(CustomQuest {
            reward: "0 Orbs".into(),
            ..sample()
        }
        .validate()
        .is_err());
        assert!(CustomQuest {
            reward: format!("{} Orbs", MAX_ORBS + 1),
            ..sample()
        }
        .validate()
        .is_err());
        assert!(CustomQuest {
            reward: "   ".into(),
            ..sample()
        }
        .validate()
        .is_err());
        assert!(CustomQuest {
            reward: "1 Gem".into(),
            ..sample()
        }
        .validate()
        .is_ok());
    }

    #[test]
    fn rejects_non_numeric_client_ids() {
        assert!(CustomQuest {
            client_id: Some("abc123".into()),
            ..sample()
        }
        .validate()
        .is_err());
        assert!(CustomQuest {
            client_id: Some("1461154307171811401".into()),
            ..sample()
        }
        .validate()
        .is_ok());
    }

    #[test]
    fn to_domain_maps_targets_and_defaults() {
        let video = sample().to_domain();
        assert_eq!(video.target, LaunchTarget::Video);
        assert_eq!(video.client_id, DEFAULT_CLIENT_ID);

        let exe = CustomQuest {
            kind: "exe".into(),
            exe_name: Some("Eve.exe".into()),
            client_id: Some("363413402300710912".into()),
            ..sample()
        }
        .to_domain();
        assert_eq!(
            exe.target,
            LaunchTarget::Exe {
                exe_name: "Eve.exe".into()
            }
        );
        assert_eq!(exe.client_id, "363413402300710912");
    }

    #[test]
    fn store_round_trips_through_a_directory() {
        let dir = std::env::temp_dir().join(format!("astral_queststore_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);

        assert!(load_from(&dir).is_empty(), "missing file reads empty");

        let quest = CustomQuest {
            id: "custom_42".into(),
            ..sample()
        };
        add_to(&dir, quest.clone()).unwrap();
        assert_eq!(load_from(&dir), vec![quest.clone()]);
        assert!(add_to(&dir, quest.clone()).is_err(), "duplicate id");

        assert!(remove_from(&dir, "custom_42").unwrap());
        assert!(!remove_from(&dir, "custom_42").unwrap());
        assert!(load_from(&dir).is_empty());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
