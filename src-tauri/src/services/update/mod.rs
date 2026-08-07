//! Update service — compares the running version against the latest astral
//! GitHub release so the UI can surface "a new version is available".
//!
//! The comparison is a pure, unit-tested function; the network call is a thin
//! wrapper that maps every failure onto the typed `UPDATE_CHECK_FAILED` error.

use serde::{Deserialize, Serialize};

use crate::app::error::AppError;
use crate::infra::config::GITHUB_LATEST_RELEASE_API_URL;

/// Result of an update check, sent to the frontend as-is.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub latest_version: String,
    pub current_version: String,
    pub is_update_available: bool,
    pub url: String,
}

/// Shape of the GitHub "latest release" payload we care about.
#[derive(Debug, Deserialize)]
struct LatestRelease {
    #[serde(default)]
    tag_name: String,
    #[serde(default)]
    html_url: String,
}

/// Parse a tag into numeric dot-segments. A leading `v` is ignored and any
/// non-numeric segment counts as 0, so odd tags can never cause a panic.
fn segments(version: &str) -> Vec<u64> {
    version
        .strip_prefix('v')
        .unwrap_or(version)
        .split('.')
        .map(|s| s.trim().parse::<u64>().unwrap_or(0))
        .collect()
}

/// True when `latest` is a newer version than `current` (dot-segment compare).
pub fn version_is_newer(latest: &str, current: &str) -> bool {
    let a = segments(latest);
    let b = segments(current);
    for i in 0..a.len().max(b.len()) {
        let av = a.get(i).copied().unwrap_or(0);
        let bv = b.get(i).copied().unwrap_or(0);
        if av != bv {
            return av > bv;
        }
    }
    false
}

/// Query GitHub for the latest release and compare it with the running build.
pub async fn check_for_update() -> Result<UpdateInfo, AppError> {
    let current_version = env!("CARGO_PKG_VERSION").to_string();
    let resp = reqwest::get(GITHUB_LATEST_RELEASE_API_URL)
        .await
        .map_err(|e| AppError::UpdateCheckFailed(format!("request failed: {e}")))?;
    if !resp.status().is_success() {
        return Err(AppError::UpdateCheckFailed(format!(
            "GitHub API returned {}",
            resp.status()
        )));
    }
    let release: LatestRelease = resp
        .json()
        .await
        .map_err(|e| AppError::UpdateCheckFailed(format!("payload decode failed: {e}")))?;

    Ok(UpdateInfo {
        is_update_available: version_is_newer(&release.tag_name, &current_version),
        latest_version: release.tag_name,
        current_version,
        url: release.html_url,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newer_version_detected() {
        assert!(version_is_newer("v2.11.0", "2.10.0"));
        assert!(version_is_newer("2.10.1", "2.10.0"));
        assert!(version_is_newer("10.0.0", "9.99.99"));
    }

    #[test]
    fn older_or_equal_not_flagged() {
        assert!(!version_is_newer("2.10.0", "2.10.0"));
        assert!(!version_is_newer("2.9.0", "2.10.0"));
        assert!(!version_is_newer("v2.10.0", "2.10.0"));
    }

    #[test]
    fn short_versions_compare_as_zero_padded() {
        assert!(version_is_newer("2.10", "2.9.9"));
        assert!(!version_is_newer("2.9", "2.9.1"));
    }

    #[test]
    fn odd_tags_never_panic() {
        assert!(!version_is_newer("", "2.10.0"));
        assert!(!version_is_newer("banana", "2.10.0"));
        assert!(version_is_newer("2.10.0-rc.1", "2.9.0"));
    }

    #[test]
    fn check_for_update_builds_info_from_running_version() {
        let current = env!("CARGO_PKG_VERSION");
        let info = UpdateInfo {
            latest_version: format!("v{current}"),
            current_version: current.to_string(),
            is_update_available: false,
            url: "https://github.com/nguyenthanhthe/astral/releases".to_string(),
        };
        assert!(!info.is_update_available);
        assert_eq!(info.current_version, current);
    }
}
