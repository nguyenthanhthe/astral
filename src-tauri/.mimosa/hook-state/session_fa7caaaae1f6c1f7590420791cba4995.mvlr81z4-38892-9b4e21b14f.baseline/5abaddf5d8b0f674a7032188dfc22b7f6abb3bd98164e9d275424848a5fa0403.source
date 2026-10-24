//! Persistent Rich Presence over Discord IPC.
//!
//! A quest session holds one RPC connection open for its whole duration and
//! re-sends `SET_ACTIVITY` periodically. This matters because Discord clears
//! a client's activity the moment its RPC socket closes — a one-shot
//! `SET_ACTIVITY` only shows for the few hundred milliseconds the socket is
//! alive, which no quest would credit. `ActivityGuard` keeps the socket open
//! and re-arms the `[start, end]` window until the session ends.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::infra::config::{DEFAULT_CLIENT_ID, QUEST_NONCE_PREFIX};
use crate::services::discord::ipc;

/// How often the holder re-sends `SET_ACTIVITY` to keep the presence armed.
const ACTIVITY_REFRESH_INTERVAL: Duration = Duration::from_secs(45);
/// Wake-up granularity so `stop()` returns promptly (<= this latency).
const ACTIVITY_WAKE_INTERVAL: Duration = Duration::from_millis(250);

pub struct ActivityRequest {
    pub client_id: String,
    pub details: String,
    pub state: String,
    pub duration_secs: u64,
}

/// Owns the held RPC connection for one session. `stop()` (also on drop)
/// sends `CLEAR_ACTIVITY` and closes the socket so no ghost presence
/// outlives the session.
pub struct ActivityGuard {
    stop: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
}

impl ActivityGuard {
    /// Signal the holder thread to clear the activity and close the socket.
    pub fn stop(&mut self) {
        if let Some(handle) = self.handle.take() {
            self.stop.store(true, Ordering::Relaxed);
            let _ = handle.join();
        }
    }
}

impl Drop for ActivityGuard {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Open a connection for the target app, handshake, arm the activity, and
/// hold the socket open (re-arming every `ACTIVITY_REFRESH_INTERVAL`) until
/// the returned guard is stopped.
pub fn hold_activity(req: ActivityRequest) -> Result<ActivityGuard, String> {
    let mut stream =
        ipc::open().map_err(|e| format!("Failed to connect to Discord IPC: {e}"))?;
    ipc::handshake(&mut stream, &req.client_id)
        .map_err(|e| format!("IPC handshake failed: {e}"))?;

    let start_ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let end_ts = start_ts + req.duration_secs;
    let pid = std::process::id();
    let session_nonce = format!("{QUEST_NONCE_PREFIX}_session");

    ipc::set_activity(
        &mut stream,
        pid,
        &req.details,
        &req.state,
        start_ts,
        end_ts,
        &session_nonce,
    )
    .map_err(|e| format!("Set activity failed: {e}"))?;

    let stop = Arc::new(AtomicBool::new(false));
    let stop_flag = stop.clone();
    let clear_nonce = format!("{QUEST_NONCE_PREFIX}_clear");

    let handle = thread::spawn(move || {
        let mut since_refresh = Duration::ZERO;
        loop {
            thread::sleep(ACTIVITY_WAKE_INTERVAL);
            if stop_flag.load(Ordering::Relaxed) {
                break;
            }
            since_refresh += ACTIVITY_WAKE_INTERVAL;
            if since_refresh >= ACTIVITY_REFRESH_INTERVAL {
                since_refresh = Duration::ZERO;
                // Keep the original start so the progress bar never resets.
                let _ = ipc::set_activity(
                    &mut stream,
                    pid,
                    &req.details,
                    &req.state,
                    start_ts,
                    end_ts,
                    &session_nonce,
                );
            }
        }
        let _ = ipc::clear_activity(&mut stream, pid, &clear_nonce);
    });

    Ok(ActivityGuard {
        stop,
        handle: Some(handle),
    })
}

/// Remove the app's Rich Presence over a short-lived IPC connection. Best
/// effort: failure (e.g. Discord closed) is swallowed by the caller.
///
/// Handshakes first — writing a frame on a raw connection makes Discord log
/// `did not handshake` and drop the socket, which would defeat the clear.
pub fn clear_activity() -> Result<(), String> {
    let mut stream = ipc::open().map_err(|e| format!("Failed to connect to Discord IPC: {e}"))?;
    ipc::handshake(&mut stream, DEFAULT_CLIENT_ID)
        .map_err(|e| format!("IPC handshake failed: {e}"))?;
    ipc::clear_activity(
        &mut stream,
        std::process::id(),
        &format!("{QUEST_NONCE_PREFIX}_clear"),
    )
    .map_err(|e| format!("Clear activity failed: {e}"))
}
