//! One-shot `SET_ACTIVITY` helper used by the session engine (and available
//! to commands) to advertise Rich Presence with a `[start, end]` window so
//! the native Discord UI shows a live progress bar.

use std::time::{SystemTime, UNIX_EPOCH};

use crate::infra::config::QUEST_NONCE_PREFIX;
use crate::services::discord::ipc;

pub struct ActivityRequest {
    pub client_id: String,
    pub details: String,
    pub state: String,
    pub duration_secs: u64,
}

/// Open a short-lived IPC connection, handshake, and send the activity.
pub fn send_activity(req: ActivityRequest) -> Result<(), String> {
    let mut stream = ipc::open().map_err(|e| format!("Failed to connect to Discord IPC: {e}"))?;
    ipc::handshake(&mut stream, &req.client_id)
        .map_err(|e| format!("IPC handshake failed: {e}"))?;

    let start_ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let end_ts = start_ts + req.duration_secs;

    ipc::set_activity(
        &mut stream,
        std::process::id(),
        &req.details,
        &req.state,
        start_ts,
        end_ts,
        &format!("{QUEST_NONCE_PREFIX}_session"),
    )
    .map_err(|e| format!("Set activity failed: {e}"))
}

/// Remove the app's Rich Presence over a short-lived IPC connection. Best
/// effort: failure (e.g. Discord closed) is swallowed by the caller.
pub fn clear_activity() -> Result<(), String> {
    let mut stream = ipc::open().map_err(|e| format!("Failed to connect to Discord IPC: {e}"))?;
    ipc::clear_activity(
        &mut stream,
        std::process::id(),
        &format!("{QUEST_NONCE_PREFIX}_clear"),
    )
    .map_err(|e| format!("Clear activity failed: {e}"))
}
