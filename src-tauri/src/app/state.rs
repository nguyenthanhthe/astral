//! Managed application state.
//!
//! Replaces the `static DETECTABLE_CACHE: Mutex<Option<...>>` (and any future
//! globals) with a single `AppState` owned by Tauri. The borrow checker and
//! the managed-state lifecycle take over from hand-rolled globals, and tests
//! can construct the state directly.

use std::collections::HashSet;
use std::sync::{Arc, RwLock};

use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tokio::sync::Notify;

use crate::services::catalog::game_catalog::Catalog;
use crate::services::session::engine::SessionTask;
use crate::DiscordStatus;

/// User-tunable settings. Phase 0 keeps this minimal; `set_settings`/`get_settings`
/// commands arrive in Phase 5.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Settings {
    /// Trim the working set once after the window renders (Windows only).
    #[serde(default)]
    pub memory_trim_on_start: bool,
}

/// Additive settings update: only the present fields are applied.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SettingsPatch {
    #[serde(default)]
    pub memory_trim_on_start: Option<bool>,
}

/// Registry of spoofer PIDs this app launched (Windows only).
///
/// Tracks exact PIDs instead of process names so `stop_spoofer` can kill only
/// what we spawned — never a user's real game.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SpooferRegistry {
    pids: HashSet<u32>,
}

impl SpooferRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, pid: u32) {
        self.pids.insert(pid);
    }

    pub fn remove(&mut self, pid: u32) {
        self.pids.remove(&pid);
    }

    pub fn clear_all(&mut self) {
        self.pids.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.pids.is_empty()
    }

    pub fn len(&self) -> usize {
        self.pids.len()
    }

    pub fn pids(&self) -> impl Iterator<Item = &u32> {
        self.pids.iter()
    }
}

pub struct AppState {
    /// Latest Discord IPC session state; every change is mirrored to the
    /// `discord://status` event (Phase 1).
    pub discord: RwLock<DiscordStatus>,
    /// Typed catalog cache (validated `DetectableGame` records) — Phase 2.
    pub catalog: RwLock<Option<Catalog>>,
    /// Currently running session (Phase 3), `None` when idle.
    pub session: RwLock<Option<crate::domain::session::Session>>,
    /// Handle to the session engine task (stop signal + abort handle).
    pub session_task: RwLock<Option<SessionTask>>,
    /// PIDs of spoofer processes we launched (Windows only).
    pub spoofer: RwLock<SpooferRegistry>,
    /// Current settings.
    pub settings: RwLock<Settings>,
    /// Handle used to `emit` events to the frontend.
    pub app_handle: AppHandle,
    /// Wakes the Discord connection task so `check_discord_session` can ask for
    /// an immediate (out-of-backoff) reconnect attempt.
    pub connect_notify: Arc<Notify>,
}

impl AppState {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            discord: RwLock::new(DiscordStatus::disconnected()),
            catalog: RwLock::new(None),
            session: RwLock::new(None),
            session_task: RwLock::new(None),
            spoofer: RwLock::new(SpooferRegistry::new()),
            settings: RwLock::new(Settings::default()),
            app_handle,
            connect_notify: Arc::new(Notify::new()),
        }
    }

    pub fn read_discord(&self) -> std::sync::RwLockReadGuard<'_, DiscordStatus> {
        self.discord.read().unwrap_or_else(|p| p.into_inner())
    }

    pub fn write_discord(&self) -> std::sync::RwLockWriteGuard<'_, DiscordStatus> {
        self.discord.write().unwrap_or_else(|p| p.into_inner())
    }

    pub fn read_catalog(&self) -> std::sync::RwLockReadGuard<'_, Option<Catalog>> {
        self.catalog.read().unwrap_or_else(|p| p.into_inner())
    }

    pub fn write_catalog(&self) -> std::sync::RwLockWriteGuard<'_, Option<Catalog>> {
        self.catalog.write().unwrap_or_else(|p| p.into_inner())
    }

    pub fn write_spoofer(&self) -> std::sync::RwLockWriteGuard<'_, SpooferRegistry> {
        self.spoofer.write().unwrap_or_else(|p| p.into_inner())
    }

    pub fn read_spoofer(&self) -> std::sync::RwLockReadGuard<'_, SpooferRegistry> {
        self.spoofer.read().unwrap_or_else(|p| p.into_inner())
    }

    pub fn read_session(
        &self,
    ) -> std::sync::RwLockReadGuard<'_, Option<crate::domain::session::Session>> {
        self.session.read().unwrap_or_else(|p| p.into_inner())
    }

    pub fn write_session(
        &self,
    ) -> std::sync::RwLockWriteGuard<'_, Option<crate::domain::session::Session>> {
        self.session.write().unwrap_or_else(|p| p.into_inner())
    }

    pub fn read_session_task(&self) -> std::sync::RwLockReadGuard<'_, Option<SessionTask>> {
        self.session_task.read().unwrap_or_else(|p| p.into_inner())
    }

    pub fn write_session_task(&self) -> std::sync::RwLockWriteGuard<'_, Option<SessionTask>> {
        self.session_task.write().unwrap_or_else(|p| p.into_inner())
    }

    pub fn read_settings(&self) -> std::sync::RwLockReadGuard<'_, Settings> {
        self.settings.read().unwrap_or_else(|p| p.into_inner())
    }

    pub fn write_settings(&self) -> std::sync::RwLockWriteGuard<'_, Settings> {
        self.settings.write().unwrap_or_else(|p| p.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_default_disables_trim() {
        assert!(!Settings::default().memory_trim_on_start);
    }

    #[test]
    fn settings_round_trip_json() {
        let s = Settings {
            memory_trim_on_start: true,
        };
        let json = serde_json::to_string(&s).unwrap();
        let back: Settings = serde_json::from_str(&json).unwrap();
        assert_eq!(back, s);
    }

    #[test]
    fn settings_patch_only_overrides_present_fields() {
        let base = Settings {
            memory_trim_on_start: false,
        };
        let patch = SettingsPatch {
            memory_trim_on_start: Some(true),
        };
        let merged = Settings {
            memory_trim_on_start: patch.memory_trim_on_start.unwrap_or(base.memory_trim_on_start),
        };
        assert!(merged.memory_trim_on_start);

        let empty = SettingsPatch::default();
        let merged = Settings {
            memory_trim_on_start: empty
                .memory_trim_on_start
                .unwrap_or(base.memory_trim_on_start),
        };
        assert!(!merged.memory_trim_on_start);
    }

    #[test]
    fn spoofer_registry_tracks_pids() {
        let mut reg = SpooferRegistry::new();
        assert!(reg.is_empty());
        reg.insert(1234);
        reg.insert(5678);
        reg.insert(1234);
        assert_eq!(reg.len(), 2);
        assert!(reg.pids().any(|p| *p == 1234));
        reg.remove(1234);
        assert_eq!(reg.len(), 1);
        assert!(!reg.pids().any(|p| *p == 1234));
    }
}
