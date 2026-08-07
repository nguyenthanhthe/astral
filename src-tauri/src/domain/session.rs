//! Session aggregate — the single source of truth for progress once the
//! frontend timer is removed (Phase 3).

use std::fmt;
use std::time::{Duration, Instant};

use super::quest::Quest;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SessionId(pub String);

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionKind {
    /// Spoof a real game executable on disk (exe quest).
    Exe,
    /// Console quest via Discord IPC activity.
    Console,
    /// Stream quest via Discord IPC activity.
    Stream,
}

#[derive(Debug, Clone)]
pub struct Session {
    pub id: SessionId,
    pub quest: Quest,
    pub kind: SessionKind,
    /// Wall-clock start; `Instant` so elapsed time is monotonic.
    pub started_at: Instant,
    /// Total wall-clock target for the quest.
    pub target_sec: Duration,
    /// Progress (0..=100) carried in from the quest when the session began.
    pub initial_percent: u8,
}

impl Session {
    pub fn progress(&self, now: Instant) -> u8 {
        let elapsed = now.saturating_duration_since(self.started_at);
        let ratio = if self.target_sec.is_zero() {
            1.0
        } else {
            (elapsed.as_secs_f64() / self.target_sec.as_secs_f64()).min(1.0)
        };
        let start = self.initial_percent.min(100) as f64;
        let pct = start + ratio * (100.0 - start);
        pct.round().clamp(0.0, 100.0) as u8
    }

    pub fn elapsed_sec(&self, now: Instant) -> u64 {
        now.saturating_duration_since(self.started_at).as_secs()
    }

    pub fn remaining_sec(&self, now: Instant) -> u64 {
        self.target_sec
            .saturating_sub(now.saturating_duration_since(self.started_at))
            .as_secs()
    }

    pub fn is_finished(&self, now: Instant) -> bool {
        now.saturating_duration_since(self.started_at) >= self.target_sec
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::target::LaunchTarget;

    fn session_with(start: u8, target_sec: u64) -> Session {
        Session {
            id: SessionId("s1".into()),
            quest: Quest::new(
                "q1",
                "title",
                "game",
                LaunchTarget::Console,
                "client",
                super::super::reward::Reward::Orbs(700),
                start,
            ),
            kind: SessionKind::Console,
            started_at: Instant::now(),
            target_sec: Duration::from_secs(target_sec),
            initial_percent: start,
        }
    }

    #[test]
    fn progress_at_start_is_initial() {
        let s = session_with(20, 900);
        assert_eq!(s.progress(Instant::now()), 20);
    }

    #[test]
    fn progress_at_half_is_midpoint() {
        let s = session_with(0, 900);
        let half = s.started_at + Duration::from_secs(450);
        assert_eq!(s.progress(half), 50);
    }

    #[test]
    fn progress_clamps_at_100() {
        let s = session_with(50, 900);
        let end = s.started_at + Duration::from_secs(900);
        let after = s.started_at + Duration::from_secs(2_000);
        assert_eq!(s.progress(end), 100);
        assert_eq!(s.progress(after), 100);
    }

    #[test]
    fn progress_blends_saved_progress() {
        let s = session_with(50, 100);
        let quarter = s.started_at + Duration::from_secs(25);
        assert_eq!(s.progress(quarter), 63);
    }

    #[test]
    fn remaining_and_finished() {
        let s = session_with(0, 60);
        assert_eq!(s.remaining_sec(s.started_at), 60);
        assert!(!s.is_finished(s.started_at + Duration::from_secs(59)));
        assert!(s.is_finished(s.started_at + Duration::from_secs(60)));
    }
}
