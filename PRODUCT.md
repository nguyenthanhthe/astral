# Astral — Product Context

> Captured via `impeccable init` from the incumbent implementation (README,
> SPEC, CHANGELOG, code). Product truth only; visual decisions live in
> DESIGN.md.

## What Astral is

Astral is a desktop companion app for **Discord's Quest & Activity program**
(Tauri 2 + React 18 + Rust). It answers one question at a glance: *"what is
running, and how far along is it?"*

## Who uses it

Discord users who want to complete Discord quests (play a game 15 minutes,
stream 15 minutes, watch a quest video 30 seconds) to earn Orb rewards.
They run it on Windows (primary), macOS and Linux, on a 960×640 (resizable)
desktop window, often alongside the Discord client and a game.

## What it actually does (product truth — never overclaim)

- **Quest list**: curated quest presets + quests the user **mirrored** from
  their own Discord client (Quest Mirror, persisted on disk, token-free),
  plus searchable Discord detectable-games catalog entries (24k+ games).
- **Sessions are honest simulations**: Astral drives progress via Rich
  Presence over Discord's local IPC (or, on Windows, a catalog-backed
  spoofer). It never reads the account token and never claims rewards on the
  user's behalf. The UI always says the session is simulated and to confirm
  the reward in Discord.
- **Demo mode**: a display-only session with no spoofer and no Discord
  traffic; finishing it shows a reward overlay explicitly captioned as
  simulated.
- **Quest targets**: real `.exe`, console (PS5/Xbox), voice stream, and quest
  video — each with its own duration contract (15 min game/stream/console,
  30 s video).
- **Unverified quests are labelled**: quests whose executable is not in
  Discord's detectable catalog say "may not count".
- **Extras**: RAM trim on start (Windows), catalog cache on disk, update
  check against GitHub releases.

## Non-negotiable constraints

- Dark-first desktop utility; compact density; nothing decorative that hides
  state.
- Every status must be readable as text (color never carries meaning alone).
- Honest copy: no invented rewards, no marketing numerals.
- Keyboard-accessible controls; real buttons/inputs only.
- Numbers that tick (countdowns, percentages, process names) render in mono.
- All network access goes through the Rust backend; the allowlist is Discord
  API + GitHub releases API only.
