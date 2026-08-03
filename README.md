# Astral 🌌

**Astral** is a high-performance, open-source Discord Activity & Quest Manager built with **Tauri v2**, **Rust**, and **React + TypeScript**.

It enables seamless game activity spoofing, custom Rich Presence, and automated Discord Orb Quest completion with minimal RAM usage (<30 MB) and direct Windows IPC pipe integration.

---

## 🌟 Key Features

- **Direct Discord IPC Connector**: Connects natively to `\\.\pipe\discord-ipc-0` without third-party webhooks or tokens.
- **Ultra-Fast Rust Core**: Low-level process runner and timer engine built with Rust for maximum efficiency.
- **Modern Glassmorphism UI**: Beautiful, responsive desktop interface crafted with React, TypeScript, and Tailwind CSS.
- **Live Detectable Game Database**: Instant search over 23,000+ Discord-detectable titles.
- **Automated Quest Timers**: Built-in countdown timers with optional auto-cleanup.
- **System Tray Ready**: Runs silently in the background while questing.

---

## 🛠️ Tech Stack

- **Desktop Framework**: [Tauri v2](https://v2.tauri.app/)
- **Backend**: Rust (`tokio`, `serde`, `winapi`)
- **Frontend**: React 18, TypeScript, Vite, Vanilla CSS
- **IPC Protocol**: Discord Local IPC (`opcode` packet handler)

---

## 🚀 Getting Started

### Prerequisites

1. **Node.js** (v18+)
2. **Rust Toolchain** (`rustup` with MSVC C++ Build Tools on Windows)

### Installation & Run

```bash
# Install dependencies
npm install

# Run in development mode
npm run tauri dev

# Build production executable (.exe)
npm run tauri build
```

---

## 📜 License

Distributed under the **MIT License**. See `LICENSE` for details.
