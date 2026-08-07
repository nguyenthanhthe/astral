//! Spoofer orchestrator (Windows).
//!
//! Replaces the old `Desktop/Win64` + `taskkill /im` approach with:
//! - executables staged under the platform app-data dir (`SPOOF_DIR_NAME`);
//! - kill **only PIDs we spawned** (tracked in `AppState.spoofer`), never by
//!   image name — so a user's real game can never be killed;
//! - simulation targets derived from the catalog's `win32_exe_names`
//!   (Discord detects a running process by that exact list), with no
//!   hardcoded `evelauncher.exe` / `WWM.exe` aliases.

pub mod orchestrator;
