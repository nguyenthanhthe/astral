# Changelog

All notable changes to this project are documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

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
