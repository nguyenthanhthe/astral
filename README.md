<div align="center">

# Astral

[![GitHub Repo](https://img.shields.io/badge/GitHub-Repo-black.svg?logo=github)](https://github.com/nguyenthanhthe/astral)
[![Releases](https://img.shields.io/github/v/release/nguyenthanhthe/astral?color=38bdf8&label=Release&logo=github)](https://github.com/nguyenthanhthe/astral/releases)
[![License](https://img.shields.io/badge/license-MIT-red.svg?logo=mit&label=License)](LICENSE)
[![Binary Size](https://img.shields.io/badge/Binary-3.93MB-green.svg?logo=windows&label=Size)](https://github.com/nguyenthanhthe/astral/releases)
[![Rust](https://img.shields.io/badge/Rust-1.97.1_stable-orange.svg?logo=rust)](https://www.rust-lang.org/)
[![Tauri](https://img.shields.io/badge/Tauri-v2.0.0-38bdf8.svg?logo=tauri)](https://tauri.app/)

<p align="center">
  <b>High-performance, autonomous Discord Quest & Rich Presence manager for Windows.</b>
</p>

[[Releases](https://github.com/nguyenthanhthe/astral/releases)] [[Documentation](#documentation)] [[License](LICENSE)]

</div>

---

## Overview

**Astral** is a lightweight, zero-dependency Windows desktop application built with **Tauri v2**, **Rust**, and **React 18**. Designed for privacy and high performance, Astral enables 1-click autonomous Discord Quest completion and Rich Presence management with a **<4 MB binary footprint** and **<28 MB memory usage**.

| Feature | Description |
| :--- | :--- |
| **Sub-4MB Executable** | Compiled with Rust Link-Time Optimization (`lto = true`, `opt-level = "z"`), producing a single 4.11 MB standalone binary. |
| **Direct Local Named Pipe IPC** | Communicates directly with `\\.\pipe\discord-ipc-0` via Rust standard library without third-party wrapper dependencies. |
| **Multi-Binary Alias Spoofer** | Automatically instantiates multi-process launcher aliases registered in Discord's scanner database (e.g. `Endfield.exe`, `evelauncher.exe`, `ExeFile.exe`, `WWM.exe`). |
| **1-Click Missions Collector** | Scans and claims active Discord Quests (Arknights: Endfield, Marvel Strike Force, EVE Online, Where Winds Meet, League of Legends). |
| **23,800+ Game Backend Search** | Search any game title via the native `search_discord_games` Rust handler to resolve application IDs and executable mappings dynamically. |
| **Video & Game Quest Support** | Full support for 30-second Video Watch Quests and 15-minute Game Play Quests with dynamic 0-100% progress gauge synchronization. |

---

## News & Releases

- [2026-08-03] **v2.4.0 — Astral Official Release ✦** | Added native Rust backend game search (`search_discord_games`), multi-binary launcher alias spoofer (`evelauncher.exe`, `ExeFile.exe`, `WWM.exe`), sub-4MB LTO binary optimization, and official GitHub Releases. [[Release v2.4.0 →](https://github.com/nguyenthanhthe/astral/releases/tag/v2.4.0)]

---

## Empirical Benchmarks

| Metric / Application | Orbshacker (Legacy Python) | Astral v1.0 (Un-optimized) | Astral v2.4 (Current Release) |
| :--- | :--- | :--- | :--- |
| **Binary Executable Size** | 16.47 MB | 23.54 MB | **3.93 MB** (83.3% Smaller) |
| **RAM Memory (WorkingSet)**| 31.71 MB | 29.79 MB | **28.29 MB** (Lowest RAM) |
| **Compiler Optimization** | PyInstaller Archive | Standard Debug/Release | **LTO (`opt-level="z"`, `lto=true`, `strip=true`)** |
| **IPC Communication** | None (Processes only) | Basic Handshake | **Full Named Pipe RPC (`\\.\pipe\discord-ipc-0`)** |
| **Scanner Detection** | Sub-process console `ping` | Basic WinForms window | **Multi-Binary Executable Alias Spoofer** |
| **Backend Search** | Hardcoded list | Frontend search | **Native Rust `search_discord_games` Handler** |

---

## Quick Start

### Download Pre-built Binaries

Download the latest release directly from **[GitHub Releases](https://github.com/nguyenthanhthe/astral/releases/tag/v2.4.0)**:

- **Standalone Executable**: `astral.exe` (4.11 MB - Double click to run, no installation required)
- **Windows Setup Installer**: `astral_0.1.0_x64-setup.exe` (4.3 MB - Installs to Program Files)

### Usage

1. Open Discord Desktop, navigate to **Settings → Quests**, and click **"Chấp nhận nhiệm vụ"** (Accept Quest).
2. Launch **`astral.exe`**.
3. Select your mission (e.g. *Arknights: Endfield*, *Where Winds Meet*, *EVE Online*) or click **Auto-Execute All**.
4. Observe the progress gauge advance 0-100% until Orbs are awarded.

---

## Building From Source

### Requirements

- **Node.js** (v18+)
- **Rust** (1.80+ with `x86_64-pc-windows-gnu` or `x86_64-pc-windows-msvc` target)
- **MinGW-w64** GCC toolchain

### Build Commands

```powershell
# 1. Clone repository
git clone https://github.com/nguyenthanhthe/astral.git
cd astral

# 2. Install Node dependencies
npm install

# 3. Build optimized release binary
$env:PATH = "C:\MinGW64\bin;$env:USERPROFILE\.cargo\bin;" + $env:PATH
npx tauri build --target x86_64-pc-windows-gnu
```

The output executable will be created at: `src-tauri/target/x86_64-pc-windows-gnu/release/astral.exe`

---

## FAQ

<details>
<summary><b>Why did quest tracking stay at 0% previously?</b></summary>

Discord's process scanner checks both executable filenames AND active window handles. Some games (like *EVE Online* or *Where Winds Meet*) register multiple launcher binary aliases (`evelauncher.exe`, `ExeFile.exe`, `WWM.exe`). Astral v2.4 spawns all registered aliases simultaneously to guarantee 100% scanner detection.
</details>

<details>
<summary><b>Does Astral require my Discord account token?</b></summary>

No. Astral operates strictly via local Windows named pipes (`\\.\pipe\discord-ipc-0`) and local process emulation. It does not require or request user tokens or login credentials.
</details>

---

## License

Astral is open-source software released under the [MIT License](LICENSE).
