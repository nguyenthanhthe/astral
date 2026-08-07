//! Catalog service — fetches Discord's detectable-applications database over
//! HTTP (replacing the PowerShell `Invoke-RestMethod` hack), validates every
//! record at the boundary, and caches the typed result in `AppState` with a
//! TTL. A background task refreshes on startup and then on the TTL interval.
//!
//! The catalog is the single source of truth for "which executables to
//! simulate": Discord's activity scanner matches the `executables` listed
//! here, so spawning those exact names is what makes arbitrary games
//! (LoL, Endfield, …) detectable — no hardcoded per-game tables.

use std::time::{Instant, SystemTime, UNIX_EPOCH};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};

use crate::app::error::AppError;
use crate::app::state::AppState;
use crate::domain::catalog::DetectableGame;
use crate::infra::config::{CATALOG_REFRESH_INTERVAL, CATALOG_URL};

/// Backend → frontend event name for catalog refreshes.
pub const EVENT_CATALOG_UPDATED: &str = "catalog://updated";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CatalogSource {
    Network,
    Cache,
}

/// Validated, typed catalog cache.
#[derive(Debug, Clone)]
pub struct Catalog {
    pub games: Vec<DetectableGame>,
    pub fetched_at: Instant,
    pub source: CatalogSource,
}

impl Catalog {
    pub fn len(&self) -> usize {
        self.games.len()
    }

    pub fn is_empty(&self) -> bool {
        self.games.is_empty()
    }

    pub fn is_fresh(&self, now: Instant) -> bool {
        now.duration_since(self.fetched_at) < CATALOG_REFRESH_INTERVAL
    }

    /// Case-insensitive substring search over game names, capped at `limit`.
    pub fn search<'a>(&'a self, query: &str, limit: usize) -> Vec<&'a DetectableGame> {
        let q = query.to_lowercase();
        self.games
            .iter()
            .filter(|g| g.name.to_lowercase().contains(&q))
            .take(limit)
            .collect()
    }

    /// Resolve a game record by exact (case-insensitive) name.
    pub fn find(&self, name: &str) -> Option<&DetectableGame> {
        self.games.iter().find(|g| g.name.eq_ignore_ascii_case(name))
    }
}

/// Payload for `catalog://updated`.
#[derive(Debug, Clone, Serialize)]
pub struct CatalogState {
    pub count: usize,
    pub source: String,
    pub at: u64,
}

/// Fetch the detectable database over HTTP.
pub async fn fetch_games() -> Result<Vec<DetectableGame>, AppError> {
    let resp = reqwest::get(CATALOG_URL)
        .await
        .map_err(|e| AppError::Internal(format!("catalog request failed: {e}")))?;
    if !resp.status().is_success() {
        return Err(AppError::Internal(format!(
            "catalog request returned HTTP {}",
            resp.status()
        )));
    }
    let body = resp
        .bytes()
        .await
        .map_err(|e| AppError::Internal(format!("catalog read failed: {e}")))?;
    parse_games(&body)
}

/// Parse + validate the detectable database body. Invalid records are
/// dropped individually; an empty result is an error.
pub fn parse_games(body: &[u8]) -> Result<Vec<DetectableGame>, AppError> {
    let values: Vec<serde_json::Value> = serde_json::from_slice(body)
        .map_err(|e| AppError::Internal(format!("catalog decode failed: {e}")))?;

    let mut games = Vec::new();
    for v in values {
        if let Some(game) = DetectableGame::from_json(&v) {
            games.push(game);
        }
    }
    if games.is_empty() {
        return Err(AppError::CatalogEmpty);
    }
    Ok(games)
}

/// Fetch fresh data, store it in `AppState`, and emit `catalog://updated`.
/// Returns the number of games cached.
pub async fn refresh(app: &AppHandle) -> Result<usize, AppError> {
    let games = fetch_games().await?;
    let count = games.len();
    {
        let state = app.state::<AppState>();
        *state.write_catalog() = Some(Catalog {
            games,
            fetched_at: Instant::now(),
            source: CatalogSource::Network,
        });
    }
    emit_updated(app, count, "network");
    log::info!("catalog updated: {count} games");
    Ok(count)
}

/// Emit `catalog://updated` with the given source.
pub fn emit_updated(app: &AppHandle, count: usize, source: &str) {
    let _ = app.emit(
        EVENT_CATALOG_UPDATED,
        CatalogState {
            count,
            source: source.to_string(),
            at: unix_now_secs(),
        },
    );
}

/// Background catalog task: fetch at startup, then refresh on the TTL.
/// Failures are logged and retried on the next cycle — the app keeps working
/// off the empty catalog + custom quests as before.
pub fn spawn(app: &AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = refresh(&app).await {
            log::warn!("initial catalog fetch failed: {}", e.log_detail());
        }
        loop {
            tokio::time::sleep(CATALOG_REFRESH_INTERVAL).await;
            if let Err(e) = refresh(&app).await {
                log::warn!("catalog refresh failed: {}", e.log_detail());
            }
        }
    });
}

fn unix_now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Real detectable-database sample for cross-module tests (catalog + spoofer).
#[cfg(test)]
pub fn fixture_bytes() -> Vec<u8> {
    include_bytes!("fixtures/detectable_sample.json").to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_body() -> Vec<u8> {
        include_bytes!("fixtures/detectable_sample.json").to_vec()
    }

    #[test]
    fn parses_real_detectable_sample() {
        let games = parse_games(&sample_body()).unwrap();
        assert_eq!(games.len(), 3);
        assert!(games.iter().any(|g| g.name == "League of Legends"));
        assert!(games.iter().any(|g| g.name == "ARKNIGHTS: ENDFIELD"));
        assert!(games.iter().any(|g| g.name == "Genshin Impact"));
    }

    #[test]
    fn drops_invalid_records_and_empty_result_errors() {
        let body = serde_json::json!([
            {"id": "1", "name": "Valid Game", "executables": [{"name": "valid.exe", "os": "win32"}]},
            {"id": "2"}, // missing name -> invalid
            "not-an-object"
        ]);
        let games = parse_games(body.to_string().as_bytes()).unwrap();
        assert_eq!(games.len(), 1);

        let err = parse_games(b"[]").unwrap_err();
        assert_eq!(err.code(), "CATALOG_EMPTY");
    }

    #[test]
    fn catalog_search_is_case_insensitive_substring() {
        let games = parse_games(&sample_body()).unwrap();
        let cat = Catalog {
            games,
            fetched_at: Instant::now(),
            source: CatalogSource::Network,
        };
        let hits: Vec<&str> = cat
            .search("leg", 10)
            .iter()
            .map(|g| g.name.as_str())
            .collect();
        assert!(hits.contains(&"League of Legends"));
        assert!(!hits.contains(&"Genshin Impact"));

        let hits = cat.search("ENDFIELD", 10);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].name, "ARKNIGHTS: ENDFIELD");
    }

    #[test]
    fn catalog_search_respects_limit() {
        let cat = Catalog {
            games: parse_games(&sample_body()).unwrap(),
            fetched_at: Instant::now(),
            source: CatalogSource::Cache,
        };
        let hits = cat.search("Impact", 0);
        assert!(hits.is_empty());
    }

    #[test]
    fn catalog_find_matches_case_insensitive() {
        let cat = Catalog {
            games: parse_games(&sample_body()).unwrap(),
            fetched_at: Instant::now(),
            source: CatalogSource::Network,
        };
        assert!(cat.find("genshin impact").is_some());
        assert!(cat.find("League of Legends").is_some());
        assert!(cat.find("nope").is_none());
    }

    #[test]
    fn freshness_respects_ttl() {
        let cat = Catalog {
            games: parse_games(&sample_body()).unwrap(),
            fetched_at: Instant::now(),
            source: CatalogSource::Network,
        };
        assert!(cat.is_fresh(Instant::now()));
        assert!(!cat.is_fresh(Instant::now() + CATALOG_REFRESH_INTERVAL));
    }

    #[test]
    fn win32_executables_are_the_simulation_targets() {
        let games = parse_games(&sample_body()).unwrap();
        let lol = games.iter().find(|g| g.name == "League of Legends").unwrap();
        let exes = lol.win32_exe_names();
        // darwin entries excluded; case-insensitive dedupe on win32 names.
        assert!(exes.iter().any(|e| e == "lol.exe"));
        assert!(exes.iter().any(|e| e == "leagueclientux.exe"));
        assert!(!exes.iter().any(|e| e.contains(".app")));
        assert_eq!(lol.primary_exe().unwrap(), "lol.exe");
    }
}

