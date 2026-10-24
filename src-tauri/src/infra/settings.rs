//! Settings persistence — `settings.json` under the platform data dir.
//!
//! Settings previously lived only in the in-memory `RwLock`, so every
//! preference reset on restart. They now round-trip through disk; a load
//! failure (missing file, corrupt JSON, unreadable dir) falls back to
//! defaults and never blocks startup.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use crate::app::error::AppError;
use crate::app::state::Settings;

pub const SETTINGS_FILE: &str = "settings.json";

/// On-disk shape. New fields must be `#[serde(default)]` so older files
/// keep loading.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct StoredSettings {
    #[serde(default)]
    pub memory_trim_on_start: bool,
    #[serde(default)]
    pub demo_mode: bool,
}

impl From<&Settings> for StoredSettings {
    fn from(s: &Settings) -> Self {
        StoredSettings {
            memory_trim_on_start: s.memory_trim_on_start,
            demo_mode: s.demo_mode,
        }
    }
}

impl From<StoredSettings> for Settings {
    fn from(s: StoredSettings) -> Self {
        Settings {
            memory_trim_on_start: s.memory_trim_on_start,
            demo_mode: s.demo_mode,
        }
    }
}

/// Load settings from disk; any failure yields the defaults.
pub fn load(app_handle: &AppHandle) -> Settings {
    std::fs::read(settings_path(app_handle))
        .ok()
        .and_then(|bytes| serde_json::from_slice::<StoredSettings>(&bytes).ok())
        .map(Settings::from)
        .unwrap_or_default()
}

/// Persist settings to disk. Failures surface as `INTERNAL` detail (logged,
/// generic message to the user): the in-memory value is already applied, so
/// only the durability guarantee is lost.
pub fn save(app_handle: &AppHandle, settings: &Settings) -> Result<(), AppError> {
    let path = settings_path(app_handle);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| AppError::Internal(format!("settings dir create failed: {e}")))?;
    }
    let stored = StoredSettings::from(settings);
    let json = serde_json::to_string_pretty(&stored)
        .map_err(|e| AppError::Internal(format!("settings serialize failed: {e}")))?;
    std::fs::write(&path, json)
        .map_err(|e| AppError::Internal(format!("settings write failed: {e}")))?;
    Ok(())
}

fn settings_path(app_handle: &AppHandle) -> PathBuf {
    app_handle
        .path()
        .app_data_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(SETTINGS_FILE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stored_settings_round_trips() {
        let stored = StoredSettings {
            memory_trim_on_start: true,
            demo_mode: true,
        };
        let json = serde_json::to_string(&stored).unwrap();
        let back: StoredSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(back, stored);
    }

    #[test]
    fn stored_settings_defaults_fill_missing_fields() {
        let back: StoredSettings = serde_json::from_str("{}").unwrap();
        assert_eq!(back, StoredSettings::default());
    }
}
