<div align="center">

# Astral

<p align="center">
  <img src="src-tauri/icons/128x128.png" alt="Astral Logo" width="100">
</p>

[![GitHub Repo](https://img.shields.io/badge/GitHub-Repo-black.svg?logo=github)](https://github.com/nguyenthanhthe/astral)
[![Releases](https://img.shields.io/github/v/release/nguyenthanhthe/astral?color=38bdf8&label=Release&logo=github)](https://github.com/nguyenthanhthe/astral/releases)
[![License](https://img.shields.io/badge/license-MIT-red.svg?logo=mit&label=License)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-stable-orange.svg?logo=rust)](https://www.rust-lang.org/)
[![Tauri](https://img.shields.io/badge/Tauri-v2-38bdf8.svg?logo=tauri)](https://tauri.app/)

<p align="center">
  <b>Desktop companion for Discord Quest completion &amp; Rich Presence management.</b>
</p>

[[Releases](https://github.com/nguyenthanhthe/astral/releases)] [[Documentation](#documentation)] [[License](LICENSE)]

</div>

---

## Overview

**Astral** is a lightweight desktop application built with **Tauri v2**, **Rust**, and **React**. It runs Discord Quest sessions and manages Rich Presence with a small, privacy-friendly binary — no account token is ever touched; everything happens through Discord's local IPC and the public detectable-games API.

| Feature | Description |
| :--- | :--- |
| **Rust backend** | Typed `services` layer (`discord`, `catalog`, `session`, `spoofer`, `memory`, `update`) with an event-driven architecture — no Python, no PowerShell. |
| **Event-driven sessions** | A single async session engine owns every quest run: it emits `session://started`, `session://progress` (1s), `session://finished`, and `session://stopped`; the UI renders engine-pushed state and re-hydrates after a reload. |
| **HTTP game catalog** | Fetches Discord's detectable-applications database via `reqwest` (openssl), caches it with a 24h TTL, and refreshes on a background task emitting `catalog://updated`. |
| **Backend game search** | Instant, case-insensitive search over the typed catalog from `search_discord_games` — resolves application IDs and executable mappings dynamically. |
| **Hardened spoofer (Windows)** | Launches staged copies of a harmless binary named after the game's catalog executables, tracks PIDs in a registry, and kills by PID (`taskkill /pid`) — no `taskkill /im`, no `Desktop/Win64`. |
| **Video & game quests** | 30-second video/console quests and 15-minute game quests with a live progress ring; console/stream quests run purely over the Discord activity IPC. |
| **Update check & GitHub link** | The header checks the GitHub releases API for a newer version and links straight to the repository. |
| **Production UI** | Dark, Discord-native design system (see [`DESIGN.md`](DESIGN.md)); accessible, responsive, with explicit loading/error/connection states. |

---

## Releases

- **v2.11.0** — Update check & GitHub link, honest (simulated) completion, no-ghost activity, catalog-verified quests, external links open in the system browser. Assets: Linux `.deb` / `.rpm`, Windows NSIS installer + MSI, macOS universal `.dmg` (unsigned, experimental). [[Release →](https://github.com/nguyenthanhthe/astral/releases/tag/v2.11.0)]

> **Update checking** is built in: the header pill compares the running version with the latest GitHub release and links to the release page when an update is available.

---

## Quick Start

### Download Pre-built Binaries

Grab the latest release from **[GitHub Releases](https://github.com/nguyenthanhthe/astral/releases/latest)**:

- **Linux (Debian/Ubuntu)**: `astral_<version>_amd64.deb` — `sudo apt install ./astral_<version>_amd64.deb`
- **Windows**: `astral_<version>_x64-setup.exe` (NSIS installer) or the `.msi`
- **macOS (experimental, unsigned)**: `astral_<version>_universal.dmg`

### Install from the terminal

One-liner installers pull the latest release and install it for you:

| OS | Command |
| :-- | :-- |
| **Linux (Ubuntu/Debian)** | `bash <(curl -sSL https://raw.githubusercontent.com/nguyenthanhthe/astral/main/install/install.sh)` |
| **macOS** | `bash <(curl -sSL https://raw.githubusercontent.com/nguyenthanhthe/astral/main/install/install.sh)` |
| **Windows** | `irm https://raw.githubusercontent.com/nguyenthanhthe/astral/main/install/install.ps1 \| iex` |

The scripts live in [`install/`](install/) — review them before running (Linux installs the `.deb` via `dpkg`, macOS copies `astral.app` into `/Applications`, Windows installs the `.msi` via `msiexec`).
### Usage

1. Open Discord Desktop and accept a quest (e.g. *Arknights: Endfield*, *Where Winds Meet*, *Fortnite*, *EVE Online*).
2. Launch **Astral**.
3. Pick a quest or click **Start first quest**.
4. Watch the live progress ring until the reward is awarded.

---

## Architecture

```
src-tauri/src
├── lib.rs          → thin command layer (start/stop session, search, catalog, update…)
├── app/            → typed error contract + managed AppState (session, catalog, spoofer, settings)
├── domain/         → pure models: Quest, Session, LaunchTarget, Reward, DetectableGame
├── infra/          → centralised config (URLs, TTLs, durations, IPC backoff)
└── services/
    ├── discord/    → IPC connection task (self-healing) + one-shot activity helper
    ├── catalog/    → HTTP detectable-games fetch + typed cache (TTL 24h)
    ├── session/    → session engine: one async task, watch-channel stop, 1s progress events
    ├── spoofer/    → Windows orchestrator: catalog exe names, staging, PID-based kill
    ├── memory/     → working-set trimmer (SetProcessWorkingSetSize, gated)
    └── update/     → GitHub latest-release check (pure version compare)
```

The frontend (`src/`) is a thin React shell: it renders engine-pushed `session://` events, subscribes to `discord://status`, and never computes its own timers or progress.

---

## Building From Source

### Requirements

- **Node.js** (v18+, npm)
- **Rust** (stable; `rustup` recommended)
- **Linux**: `libwebkit2gtk-4.1-dev`, `libappindicator3-dev`, `librsvg2-dev`, `patchelf`

### Build Commands

```bash
# 1. Clone repository
git clone https://github.com/nguyenthanhthe/astral.git
cd astral

# 2. Install frontend dependencies
npm install

# 3. Run in development (hot-reload)
npm run tauri dev

# 4. Build a release bundle (.deb / .rpm / AppImage)
npm run tauri build
```

The bundles land in `src-tauri/target/release/bundle/`.

> **Windows builds** are produced in CI (`Release (Windows)` workflow on `v*` tags) because the app cannot be cross-compiled from Linux; the bundled `.exe`/`.msi` are attached to each release.

---

## FAQ

<details>
<summary><b>Does Astral require my Discord account token?</b></summary>

No. Astral operates strictly via Discord's local IPC (named pipe on Windows, Unix socket elsewhere) and the public detectable-games API. It never requests or stores login credentials.
</details>

<details>
<summary><b>How does quest tracking actually work?</b></summary>

For game quests, Astral spawns a staged, harmless process whose filename matches the game's executables from Discord's own detectable database, so Discord's process scanner recognises the "running" game. Console/stream quests only need the activity over the local IPC — no process at all.
</details>

<details>
<summary><b>Why do sessions now run in the backend?</b></summary>

Since v2.10.0 a dedicated async session engine owns every quest run and pushes progress events every second. The UI no longer keeps a timer, so progress stays accurate across reloads and window churn.
</details>

<details>
<summary><b>Why is the macOS build unsigned / experimental?</b></summary>

There is no local Mac or Apple Developer ID for signing. The universal `.dmg` is built by the `Release (macOS experimental)` workflow; first launch needs "Open anyway" or `sudo xattr -dr com.apple.quarantine /Applications/Astral.app`. The process spoofer is Windows-only.
</details>

---

## Documentation

- [`DESIGN.md`](DESIGN.md) — brand contract and design tokens.
- [`tasks/system-design.md`](tasks/system-design.md) — architecture and the IPC contract (commands + events).
- [`CHANGELOG.md`](CHANGELOG.md) — release history.

---

## License

Astral is open-source software released under the [MIT License](LICENSE).
