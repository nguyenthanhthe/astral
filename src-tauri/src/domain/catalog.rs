//! Catalog models. Data sourced from Discord's public
//! `applications/detectable` endpoint is **untrusted**: every record is
//! validated at parse time and malformed entries are skipped instead of
//! letting `serde_json::Value` leak into domain logic.
//!
//! The `executables` array is the authoritative list of process names
//! Discord's activity scanner matches for each game — so it is also the
//! authoritative list of what the spoofer should simulate. No hardcoded
//! per-game alias tables.

use serde::{Deserialize, Serialize};

/// One detectable process for a game (e.g. `endfield.exe`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DetectableExe {
    /// Normalised process basename (path segments stripped), e.g. `Eve.exe`.
    pub name: String,
    /// `true` for launcher processes (e.g. `leagueclientux.exe`).
    pub is_launcher: bool,
    /// Target OS for this executable (`"win32"`, `"darwin"`, …). May be empty
    /// on older records, in which case it is treated as platform-agnostic.
    pub os: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DetectableGame {
    pub name: String,
    pub client_id: String,
    pub executables: Vec<DetectableExe>,
}

impl DetectableGame {
    /// Parse one record from the detectable endpoint, skipping malformed
    /// entries. `name` and `id` are required; `executables` is optional.
    pub fn from_json(value: &serde_json::Value) -> Option<Self> {
        let name = value.get("name").and_then(|n| n.as_str())?.trim();
        if name.is_empty() {
            return None;
        }
        let client_id = value.get("id").and_then(|i| i.as_str())?.trim();
        if client_id.is_empty() {
            return None;
        }

        let mut executables = Vec::new();
        if let Some(list) = value.get("executables").and_then(|e| e.as_array()) {
            for ex in list {
                let ex_name = ex.get("name").and_then(|n| n.as_str()).unwrap_or("");
                let ex_name = ex_name.trim();
                if !ex_name.ends_with(".exe") {
                    continue;
                }
                let base = ex_name.rsplit(['/', '\\']).next().unwrap_or(ex_name);
                let is_launcher = ex
                    .get("is_launcher")
                    .and_then(|b| b.as_bool())
                    .unwrap_or(false);
                let os = ex
                    .get("os")
                    .and_then(|o| o.as_str())
                    .unwrap_or("")
                    .to_string();
                executables.push(DetectableExe {
                    name: base.to_string(),
                    is_launcher,
                    os,
                });
            }
        }

        Some(DetectableGame {
            name: name.to_string(),
            client_id: client_id.to_string(),
            executables,
        })
    }

    /// Windows process names Discord's scanner matches for this game,
    /// deduplicated case-insensitively and filtered to `win32` (or
    /// platform-agnostic) executables.
    ///
    /// This is what the spoofer should simulate — the same names Discord uses
    /// to detect the game, so detection is complete without inventing names.
    pub fn win32_exe_names(&self) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for ex in &self.executables {
            if !ex.os.is_empty() && ex.os != "win32" {
                continue;
            }
            if !ex.name.to_lowercase().ends_with(".exe") {
                continue;
            }
            let lower = ex.name.to_lowercase();
            if out.iter().any(|o: &String| o.to_lowercase() == lower) {
                continue;
            }
            out.push(ex.name.clone());
        }
        out
    }

    /// The primary (non-launcher preferred) win32 executable for display.
    pub fn primary_exe(&self) -> Option<String> {
        let names = self.win32_exe_names();
        names
            .iter()
            .find(|n| {
                self.executables
                    .iter()
                    .any(|e| e.name == **n && !e.is_launcher)
            })
            .cloned()
            .or_else(|| names.first().cloned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_record() {
        let v = serde_json::json!({
            "name": "Genshin Impact",
            "id": "12345",
            "executables": [
                {"name": "GenshinImpact.exe", "is_launcher": false, "os": "win32"},
                {"name": "launcher.exe", "is_launcher": true, "os": "win32"}
            ]
        });
        let game = DetectableGame::from_json(&v).unwrap();
        assert_eq!(game.name, "Genshin Impact");
        assert_eq!(game.client_id, "12345");
        assert_eq!(game.executables.len(), 2);
        assert_eq!(game.executables[0].name, "GenshinImpact.exe");
        assert!(!game.executables[0].is_launcher);
        assert!(game.executables[1].is_launcher);
    }

    #[test]
    fn skips_record_without_name() {
        let v = serde_json::json!({"id": "12345"});
        assert!(DetectableGame::from_json(&v).is_none());
    }

    #[test]
    fn skips_record_without_id() {
        let v = serde_json::json!({"name": "No ID"});
        assert!(DetectableGame::from_json(&v).is_none());
    }

    #[test]
    fn skips_empty_name() {
        let v = serde_json::json!({"name": "   ", "id": "9"});
        assert!(DetectableGame::from_json(&v).is_none());
    }

    #[test]
    fn normalizes_windows_paths() {
        let v = serde_json::json!({
            "name": "Game",
            "id": "1",
            "executables": [{"name": "C:\\Some\\Dir\\Game.exe", "os": "win32"}]
        });
        let game = DetectableGame::from_json(&v).unwrap();
        assert_eq!(game.executables[0].name, "Game.exe");
    }

    #[test]
    fn ignores_non_exe_entries() {
        let v = serde_json::json!({
            "name": "Game",
            "id": "1",
            "executables": [
                {"name": "Game.exe", "os": "win32"},
                {"name": "Game.app", "os": "darwin"},
                {"name": "launcher", "os": "win32"}
            ]
        });
        let game = DetectableGame::from_json(&v).unwrap();
        assert_eq!(game.executables.len(), 1);
        assert_eq!(game.executables[0].name, "Game.exe");
    }

    #[test]
    fn win32_exes_filter_macos_and_dedupe() {
        let v = serde_json::json!({
            "name": "League of Legends",
            "id": "1402418696126992445",
            "executables": [
                {"name": "league of legends.app", "os": "darwin"},
                {"name": "garenaloltw/gamedata/apps/loltw/lol.exe", "os": "win32"},
                {"name": "garenalolth/gamedata/apps/lolth/lolex.exe", "os": "win32"},
                {"name": "league of legends.exe", "os": "win32"},
                {"name": "LOL.EXE", "os": "win32"},
                {"name": "leagueclientux.exe", "is_launcher": true, "os": "win32"}
            ]
        });
        let game = DetectableGame::from_json(&v).unwrap();
        let names = game.win32_exe_names();
        // darwin .app excluded; LOL.EXE deduped against lol.exe (case-insensitive)
        assert_eq!(
            names,
            vec![
                "lol.exe",
                "lolex.exe",
                "league of legends.exe",
                "leagueclientux.exe"
            ]
        );
    }

    #[test]
    fn primary_exe_prefers_non_launcher() {
        let v = serde_json::json!({
            "name": "LoL",
            "id": "1",
            "executables": [
                {"name": "leagueclientux.exe", "is_launcher": true, "os": "win32"},
                {"name": "league of legends.exe", "is_launcher": false, "os": "win32"}
            ]
        });
        let game = DetectableGame::from_json(&v).unwrap();
        assert_eq!(
            game.primary_exe(),
            Some("league of legends.exe".to_string())
        );
    }

    #[test]
    fn primary_exe_falls_back_when_only_launcher() {
        let v = serde_json::json!({
            "name": "Launcher Only",
            "id": "2",
            "executables": [{"name": "storelauncher.exe", "is_launcher": true, "os": "win32"}]
        });
        let game = DetectableGame::from_json(&v).unwrap();
        assert_eq!(game.primary_exe(), Some("storelauncher.exe".to_string()));
    }
}
