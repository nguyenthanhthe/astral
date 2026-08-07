//! Session engine — the single source of truth for progress (Phase 3).
//!
//! One tokio task owns the running session: it launches the simulation
//! (spoofer for exe quests, Discord IPC activity for console/stream quests),
//! emits `session://progress` every second, and terminates with
//! `session://finished` (time elapsed) or `session://stopped` (user stop or
//! launch error). The frontend only listens to these events — no more timer.

pub mod engine;
