# Changelog

All notable changes to this project are documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [Unreleased]

### Update check, GitHub link & README rewrite (local, no release)
- Added `services/update/mod.rs`: `check_for_update` command queries the GitHub latest-release API and reports `UpdateInfo { latest_version, current_version, is_update_available, url }`; version comparison is a pure, unit-tested dot-segment function (`v` prefix tolerant, non-numeric segments → 0). Failures map to the new typed `UPDATE_CHECK_FAILED` error whose message never leaks internals
- Header now shows a check-for-updates pill (idle → checking → up to date / new version available with a link to the release page) plus a GitHub logo that opens the repository via the shell plugin; version badge switched to the GitHub mark
- Rewrote `README.md` to match the current architecture (services layer, session engine, release assets, FAQ)

## [2.10.0] - 2026-08-07

### Backend Refactor — Phase 5 (Session engine & settings contract)
- Added `services/session/engine.rs`: a single tokio task owns the session; `start_session` returns `SESSION_ACTIVE` when already running, `stop_session` is idempotent (watch channel → clean stop with reason `USER`), and `get_session_status` re-hydrates the UI after a reload. The engine emits `session://started`, `session://progress` every 1s (`MissedTickBehavior::Skip`), `session://finished`, and `session://stopped`; the frontend dropped its `setInterval` and now renders engine-pushed progress
- Added `services/discord/activity.rs`: one-shot SET_ACTIVITY helper with `[start, end]` timestamps and `{QUEST_NONCE_PREFIX}_session` nonce; activity requests carry owned `String`s so the payload is `Send` across task boundaries
- Added `services/spoofer/orchestrator.rs` (Windows): sanitises the exe name (basename, both separators, blocks path traversal), stages a copy of `powershell.exe` inside `app_data_dir/spoof`, launches it hidden, registers PIDs in `SpooferRegistry`, and `stop_all` kills by registered PID (`taskkill /f /pid`) then removes the staging dir — no more `taskkill /im`, no more `Desktop/Win64`. `exe_names_for_simulation` derives names from the typed catalog with a `<game>.exe` fallback; non-Windows returns `PLATFORM_UNSUPPORTED`
- Added `services/memory/trimmer.rs`: the `SetProcessWorkingSetSize` FFI moved out of `lib.rs` into a gated module; `optimize_ram` now calls it
- Added `set_settings` / `get_settings` commands with an additive `SettingsPatch` (option fields only); FE wrappers `getSettings`/`setSettings` added
- `lib.rs` is now a thin command layer: `start_session`, `stop_session`, `get_session_status`, `set_settings`, `get_settings`, `refresh_catalog`, `optimize_ram`; removed the old `spoof_non_exe_quest`, `set_discord_activity`, `start_spoofer`/`stop_spoofer` and `Desktop/Win64` helpers. `quest_from_wire` converts marker quests into typed targets; the `dirs` dependency was dropped
- App state: `AppState.session` + `session_task` (stop signal + join handle), poison-tolerant `read_settings`/`write_settings`, and `SpooferRegistry.clear_all` on stop
- Frontend migrated to the event contract: `quest.ts`/`tauri.ts` now expose `startSession`/`stopSession`/`getSessionStatus` and the `SessionStarted/Progress/Finished/Stopped` wire types (client-side progress math removed); `App.tsx` is fully event-driven with `getSessionStatus()` re-hydration and keepalive/discard-safe listeners
- Tests: 68 Rust + 5 Vitest, clippy `-D warnings` clean, `npm run build` clean; live-verified boot (Discord IPC connected, `catalog updated: 23907 games`)

### Backend Refactor — Phase 2 (HTTP catalog, PowerShell removed)
- Added `services/catalog/game_catalog.rs`: fetches Discord's detectable database over HTTP (`reqwest`, `default-tls`/openssl — aws-lc-rs avoided because nasm is unavailable), validates every record into typed `DetectableGame`, caches in `AppState` as a `Catalog` with a 24h TTL, and re-fetches on a background TTL loop emitting `catalog://updated`
- Removed the PowerShell `Invoke-RestMethod` prefetch (`preload_detectable_cache`) entirely; startup now spawns the catalog task via `game_catalog::spawn` in `setup()`
- Search runs over the typed catalog: `Catalog::search` (case-insensitive substring, capped at `SEARCH_LIMIT`), dedupe vs active quests via pure `merge_catalog_hits`, and existing custom-quest fallback preserved; new `refresh_catalog` command returns `Result<CatalogState, AppError>`
- `parse_games` split out from the HTTP path for testability; real-device fixture `services/catalog/fixtures/detectable_sample.json` (League of Legends / ARKNIGHTS: ENDFIELD / Genshin Impact) added; 57 tests pass, clippy `-D warnings` clean
- Verified live: log shows `catalog updated: 23907 games` (matches the endpoint), no PowerShell anywhere; Discord IPC still connects

### Backend Refactor — Phase 1 (Discord connectivity)
- Moved Discord IPC into a `services` layer (`services/discord/ipc.rs` + `connection.rs`); added `ReadWrite: Send` so connections can cross async boundaries
- Built all IPC payloads with `serde_json::json!` (handshake + SET_ACTIVITY), eliminating the `format!`-assembled JSON that could emit invalid frames for titles containing quotes/newlines; handshake now returns a typed `HandshakeResult { username, user_id }`
- Added a self-healing connection manager: one tokio task connects, handshakes, holds the socket to detect drops, and retries with exponential backoff (200ms → 10s, capped) using `spawn_blocking` for I/O; every state change is emitted as `discord://status`
- `check_discord_session` now reads the managed state and wakes the connection task for an immediate retry instead of probing the pipe itself; catalog prefetch moved to app startup (`setup()`) and uncoupled from the session check
- Frontend subscribes to `discord://status` so the connection pill stays live without polling (timer/session migration still pending, T9)

### Backend Refactor — Phase 0 (Rust-native foundations)
- Added typed error contract `app::error::AppError` with machine-readable codes (`DISCORD_NOT_REACHABLE`, `SESSION_ACTIVE`, …) serialized as `{ code, message }`; `Internal` failures are logged with detail but never leaked to the UI
- Introduced managed `AppState` (Tauri `State` + `RwLock`) replacing the global `static DETECTABLE_CACHE`; registered once in `setup()`, with poison-tolerant accessors and PID-tracking `SpooferRegistry` and `Settings` (RAM trim off by default)
- Added `infra::config` centralising magic values (default client_id, catalog URL/TTL, quest durations, search limit, IPC backoff, spoof staging dir name)
- Added pure `domain` models — `LaunchTarget` (Exe/Console/Stream), `Quest`, `Reward`, `Session` (single progress/remaining/finished source), `DetectableGame` (validated, untrusted-catalog boundary) — replacing string markers in domain logic; the wire `DiscordQuest` markers (`[Console Quest]`/`[Stream Quest]`) are now only produced at the boundary projection so the frontend keeps working unchanged
- Removed global `Mutex` cache in favour of `AppState.catalog`; catalog writes now validate each record via `DetectableGame::from_json`
- Changed license/authors from "Daniel Pires / Strykey" to "nguyenthanhthe"

### Frontend Redesign (Production UI)
- Added [`DESIGN.md`](DESIGN.md) brand contract and CSS design tokens (`src/styles/tokens.css`) following the open-design (nexu-io/open-design) philosophy; all components now consume tokens instead of hardcoded hex values
- Rebuilt the UI with accessible primitives: every quest row is a real `<button>`, icon-only controls carry `aria-label`, session messages use `aria-live="polite"`, and the progress ring exposes `role="progressbar"`
- Added explicit loading (skeleton), error (retry), empty, and connection (checking / connected / disconnected) states
- Replaced marketing copy with production wording: removed "Celestial Edition", "23,800+", "Auto-Execute All", "Spoofing stopped", and misleading completion claims
- App version now rendered from `@tauri-apps/api/app#getVersion` instead of a hardcoded string
- Dropped the Google Fonts network dependency (system font stack) and tightened the CSP in `tauri.conf.json` accordingly
- Split `App.tsx` into focused components (`AppHeader`, `QuestList`, `SessionPanel`, `ProgressRing`, `SearchInput`, `Button`, `StatusPill`); behavior and all Rust commands unchanged
- Added `questTargetLabel` helper (+ tests) for human-readable console/stream quest targets

## [2.9.0] - 2026-08-07

### Cross-platform / macOS
- Added Unix domain socket support for Discord IPC on macOS/Linux (`discord_ipc::unix_socket_path`), using a common `ReadWrite` stream trait shared with the Windows named pipe — verified against live Discord on Linux
- Locked the process spoofer (`start_spoofer`/`stop_spoofer`) to Windows only; other platforms now return a clear error instead of running Windows-only logic
- Added experimental unsigned macOS release workflow (`release-macos.yml`) producing a universal `.dmg`/`.app` via GitHub Actions (no Apple Developer ID, no runtime testing on a physical Mac)
- Changed bundle identifier from `com.astral.app` to `com.astral.desktop` (a `.app`-suffixed identifier conflicts with the macOS bundle extension)
- Added Linux `.deb` packaging with live Discord testing on Linux

### Quality & Reliability
- Added shared Discord IPC module (`discord_ipc.rs`) replacing three duplicated handshake/SET_ACTIVITY code paths
- Removed panic-prone `unwrap()`/`expect()` calls in production paths; malformed IPC frames are now rejected gracefully instead of crashing
- Guarded against oversized IPC payloads (memory exhaustion risk)
- Added structured logging via `log` + `env_logger` (info/warn/debug across commands)
- Removed hardcoded personal fallbacks (`telecom.no1`, `C:\Users\Admin\Desktop`)

### Security
- Added Content Security Policy (production + dev) to `tauri.conf.json`

### Frontend
- Typed Tauri invoke wrappers (`src/lib/tauri.ts`) replacing `window.__TAURI_INTERNALS__` checks
- Extracted pure quest duration/progress helpers (`src/lib/quest.ts`)
- Added `ErrorBoundary` to prevent silent blank windows on render errors
- Fixed misleading default "Discord Active" state when no session is detected
- Replaced broken favicon reference

### Testing
- Added Rust unit tests (IPC framing, quest search matching, exe naming)
- Added frontend unit tests (`vitest`) for quest helpers
- Added `test` script (`vitest run`)

### CI / Build
- Added GitHub Actions workflow (`ci.yml`) running lint, tests, and builds
- Removed release binaries (`astral.exe`, `WebView2Loader.dll`) from version control
- Added `.gitignore` entries for binaries and platform-generated schemas

[2.9.0]: https://github.com/nguyenthanhthe/astral/releases/tag/v2.9.0
[2.10.0]: https://github.com/nguyenthanhthe/astral/releases/tag/v2.10.0
