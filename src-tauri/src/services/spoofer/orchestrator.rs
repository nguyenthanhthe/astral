//! Windows process-simulation orchestrator.
//!
//! The Discord activity scanner detects a game by the exact executables in
//! the detectable catalog. To make a quest "running", we stage a copy of a
//! real always-alive executable (PowerShell hosting a hidden WinForms form)
//! under each required name and spawn it. Every spawned PID is registered so
//! cleanup is exact and scoped — we never `taskkill /im` and never touch a
//! user's real game.

#[cfg(target_os = "windows")]
use std::path::PathBuf;

use tauri::AppHandle;
#[cfg(target_os = "windows")]
use tauri::Manager;

#[cfg(target_os = "windows")]
use crate::app::state::AppState;
#[cfg(target_os = "windows")]
use crate::infra::config::SPOOF_DIR_NAME;
use crate::app::error::AppError;
use crate::services::catalog::game_catalog::Catalog;

/// Resolve the executable names to simulate for a game.
///
/// Source of truth is the catalog record (its win32 executable list — the
/// exact names Discord's scanner matches). If the game isn't in the catalog,
/// fall back to `<game>.exe`.
pub fn exe_names_for_simulation(catalog: Option<&Catalog>, game_name: &str) -> Vec<String> {
    let mut names: Vec<String> = catalog
        .and_then(|c| c.find(game_name))
        .map(|g| g.win32_exe_names())
        .unwrap_or_default();
    if names.is_empty() {
        names.push(sanitize_exe_name(game_name));
    }
    names
}

/// Normalize an executable name to a safe basename ending in `.exe`.
///
/// The catalog stores full paths (e.g. `garenaloltw/gamedata/apps/loltw/lol.exe`);
/// Discord matches by basename, and we must not allow path traversal into the
/// staging dir. Handles both `/` and `\` separators.
pub fn sanitize_exe_name(name: &str) -> String {
    let base = name
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(name)
        .to_string();
    ensure_exe_suffix(&base)
}

/// Normalize an executable name to end in `.exe`.
pub fn ensure_exe_suffix(exe_name: &str) -> String {
    if exe_name.to_lowercase().ends_with(".exe") {
        exe_name.to_string()
    } else {
        format!("{}.exe", exe_name)
    }
}

/// Stage + launch one simulated process per target executable. Registers
/// every PID in `AppState.spoofer` and returns them. Non-Windows: rejected.
pub fn spawn_exe_simulation(
    app: &AppHandle,
    exe_names: &[String],
    title: &str,
) -> Result<Vec<u32>, AppError> {
    #[cfg(target_os = "windows")]
    {
        spawn_exe_simulation_windows(app, exe_names, title)
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = (app, exe_names, title);
        Err(AppError::PlatformUnsupported)
    }
}

/// Kill every PID we spawned and remove the staging dir. Safe no-op when
/// nothing is running. Non-Windows: no-op.
pub fn stop_all(app: &AppHandle) {
    #[cfg(target_os = "windows")]
    {
        stop_all_windows(app);
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = app;
    }
}

#[cfg(target_os = "windows")]
fn spawn_exe_simulation_windows(
    app: &AppHandle,
    exe_names: &[String],
    title: &str,
) -> Result<Vec<u32>, AppError> {
    use std::os::windows::process::CommandExt;
    use std::{fs, process::Command};

    const CREATE_NO_WINDOW: u32 = 0x08000000;

    let staging = spoof_dir_path(app)?;
    fs::create_dir_all(&staging).map_err(|e| AppError::Internal(format!("staging dir: {e}")))?;

    let ps_path = PathBuf::from(r"C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe");
    if !ps_path.exists() {
        return Err(AppError::Internal(
            "Windows PowerShell not found".to_string(),
        ));
    }

    let script = format!(
        "Add-Type -AssemblyName System.Windows.Forms; $f = New-Object System.Windows.Forms.Form; $f.Text = '{}'; [System.Windows.Forms.Application]::Run($f)",
        title
    );

    let mut pids = Vec::new();
    for exe in exe_names {
        let staged = staging.join(sanitize_exe_name(exe));
        if let Err(e) = fs::copy(&ps_path, &staged) {
            log::warn!("failed to stage {}: {e}", staged.display());
            continue;
        }
        match Command::new(&staged)
            .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", &script])
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
        {
            Ok(child) => {
                let pid = child.id();
                app.state::<AppState>().write_spoofer().insert(pid);
                pids.push(pid);
            }
            Err(e) => log::warn!("failed to spawn staged {}: {e}", staged.display()),
        }
    }

    if pids.is_empty() {
        return Err(AppError::Internal(
            "no spoofer process could be launched".to_string(),
        ));
    }
    log::info!("spoofer launched {} simulated PIDs", pids.len());
    Ok(pids)
}

#[cfg(target_os = "windows")]
fn stop_all_windows(app: &AppHandle) {
    use std::os::windows::process::CommandExt;
    use std::process::Command;

    const CREATE_NO_WINDOW: u32 = 0x08000000;

    let pids: Vec<u32> = app
        .state::<AppState>()
        .read_spoofer()
        .pids()
        .copied()
        .collect();
    for pid in &pids {
        let _ = Command::new("taskkill")
            .args(["/f", "/pid", &pid.to_string()])
            .creation_flags(CREATE_NO_WINDOW)
            .output();
    }
    app.state::<AppState>().write_spoofer().clear_all();

    if let Ok(dir) = spoof_dir_path(app) {
        let _ = std::fs::remove_dir_all(&dir);
    }
    log::info!("spoofer orchestrator cleaned {} PIDs", pids.len());
}

/// Resolve the app-data staging dir (`<data_dir>/spoof`).
#[cfg(target_os = "windows")]
fn spoof_dir_path(app: &AppHandle) -> Result<PathBuf, AppError> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Internal(format!("app data dir: {e}")))?;
    Ok(dir.join(SPOOF_DIR_NAME))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_takes_basename_only() {
        assert_eq!(
            sanitize_exe_name("garenaloltw/gamedata/apps/loltw/lol.exe"),
            "lol.exe"
        );
        assert_eq!(
            sanitize_exe_name(r"garenalolth\gamedata\apps\lolth\lolex.exe"),
            "lolex.exe"
        );
    }

    #[test]
    fn sanitize_blocks_path_traversal() {
        assert_eq!(sanitize_exe_name("../../evil.exe"), "evil.exe");
    }

    #[test]
    fn ensure_exe_suffix_normalizes() {
        assert_eq!(ensure_exe_suffix("Eve"), "Eve.exe");
        assert_eq!(ensure_exe_suffix("eve.exe"), "eve.exe");
        assert_eq!(ensure_exe_suffix("GAME.EXE"), "GAME.EXE");
    }

    #[test]
    fn exe_names_fallback_to_game_name() {
        let names = exe_names_for_simulation(None, "Where Winds Meet");
        assert_eq!(names, vec!["Where Winds Meet.exe".to_string()]);
    }

    #[test]
    fn exe_names_come_from_catalog() {
        let games = crate::services::catalog::game_catalog::parse_games(
            &crate::services::catalog::game_catalog::fixture_bytes(),
        )
        .unwrap();
        let catalog = Catalog {
            games,
            fetched_at: std::time::Instant::now(),
            source: crate::services::catalog::game_catalog::CatalogSource::Network,
        };
        let names = exe_names_for_simulation(Some(&catalog), "League of Legends");
        // darwin entries excluded, win32 basenames deduped
        assert!(names.iter().any(|n| n == "lol.exe"));
        assert!(names.iter().any(|n| n == "leagueclientux.exe"));
        assert!(!names.iter().any(|n| n.contains(".app")));
        assert!(!names.iter().any(|n| n.contains('/')));
    }
}
