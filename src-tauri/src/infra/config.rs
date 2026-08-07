//! Centralised configuration & magic-string elimination.
//!
//! Previously these values were scattered across `lib.rs` (and duplicated in
//! the frontend). Keeping them here gives a single source of truth that can
//! later be overridden by user settings without touching command code.

use std::time::Duration;

/// Default client_id used for custom quests and the activity fallback.
pub const DEFAULT_CLIENT_ID: &str = "356875221078245376";

/// Namespace prefix for SET_ACTIVITY nonces.
pub const QUEST_NONCE_PREFIX: &str = "astral";

/// Discord detectable-applications catalog endpoint (single allowed host).
pub const CATALOG_URL: &str = "https://discord.com/api/v9/applications/detectable";

/// How long a fetched catalog is considered fresh before refetch.
pub const CATALOG_REFRESH_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);

/// Target duration for console/stream ("video") quests.
pub const VIDEO_QUEST_DURATION_SEC: u64 = 30;

/// Target duration for real-game quests.
pub const GAME_QUEST_DURATION_SEC: u64 = 15 * 60;

/// Default reward advertised on catalog-sourced quests.
pub const DEFAULT_REWARD_ORBS: u32 = 700;

/// Cap on search results returned to the frontend.
pub const SEARCH_LIMIT: usize = 25;

/// Discord IPC reconnect backoff sequence (milliseconds), reset on success.
pub const IPC_RECONNECT_BACKOFF_MS: &[u64] = &[200, 500, 1_000, 2_000, 5_000, 10_000];

/// Subdirectory under the platform data dir where spoof executables are
/// staged. Replaces the old `Desktop/Win64` hack.
pub const SPOOF_DIR_NAME: &str = "spoof";

/// Astral repository homepage (frontend GitHub link target).
pub const GITHUB_REPO_URL: &str = "https://github.com/nguyenthanhthe/astral";

/// GitHub API endpoint for the latest release (update checks).
pub const GITHUB_LATEST_RELEASE_API_URL: &str =
    "https://api.github.com/repos/nguyenthanhthe/astral/releases/latest";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_interval_is_positive() {
        assert!(CATALOG_REFRESH_INTERVAL > Duration::ZERO);
    }

    #[test]
    fn backoff_sequence_is_increasing_and_capped() {
        assert_eq!(IPC_RECONNECT_BACKOFF_MS.first(), Some(&200));
        assert_eq!(IPC_RECONNECT_BACKOFF_MS.last(), Some(&10_000));
        for w in IPC_RECONNECT_BACKOFF_MS.windows(2) {
            assert!(w[0] < w[1]);
        }
    }

    #[test]
    fn quest_durations_match_contract() {
        assert_eq!(VIDEO_QUEST_DURATION_SEC, 30);
        assert_eq!(GAME_QUEST_DURATION_SEC, 15 * 60);
    }
}
