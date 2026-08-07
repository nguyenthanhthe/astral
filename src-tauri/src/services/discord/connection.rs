//! Self-healing Discord IPC connection manager.
//!
//! A single tokio task owns the monitoring connection to the local Discord
//! client. It connects → handshakes → holds the socket open to detect drops →
//! disconnects → retries with exponential backoff (200ms … 10s). Every state
//! change is mirrored to `AppState.discord` and emitted as `discord://status`
//! so the frontend pill stays live without polling.
//!
//! I/O runs on `spawn_blocking` (the socket API is std-blocking); the task
//! itself stays async so backoff and wake-ups never block the runtime.

use std::time::Duration;

use tauri::{AppHandle, Emitter, Manager};

use crate::app::state::AppState;
use crate::infra::config::{DEFAULT_CLIENT_ID, IPC_RECONNECT_BACKOFF_MS};
use crate::services::discord::ipc;
use crate::DiscordStatus;

/// Backend → frontend event name for Discord connection state changes.
pub const EVENT_DISCORD_STATUS: &str = "discord://status";

/// Start the connection manager task.
pub fn spawn(app: &AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn(run(app));
}

async fn run(app: AppHandle) {
    let connect_notify = app.state::<AppState>().connect_notify.clone();
    let mut failures = 0usize;

    loop {
        let connected = connect_and_watch(&app).await;

        let delay_ms = if connected {
            failures = 0;
            // Back off after every drop so we don't hammer the pipe.
            IPC_RECONNECT_BACKOFF_MS[0]
        } else {
            failures += 1;
            let idx = failures
                .saturating_sub(1)
                .min(IPC_RECONNECT_BACKOFF_MS.len() - 1);
            IPC_RECONNECT_BACKOFF_MS[idx]
        };
        log::debug!("discord reconnect in {delay_ms}ms (connected={connected})");

        tokio::select! {
            _ = tokio::time::sleep(Duration::from_millis(delay_ms)) => {}
            _ = connect_notify.notified() => {
                log::debug!("discord reconnect woken early by check_discord_session");
            }
        }
    }
}

/// One connection lifecycle: open + handshake, then block on reads until the
/// socket dies. Returns `true` if a session was ever established.
async fn connect_and_watch(app: &AppHandle) -> bool {
    let open = tauri::async_runtime::spawn_blocking(
        || -> std::io::Result<(Box<dyn ipc::ReadWrite>, ipc::HandshakeResult)> {
            let mut stream = ipc::open()?;
            let hs = ipc::handshake(&mut stream, DEFAULT_CLIENT_ID)?;
            Ok((stream, hs))
        },
    )
    .await;

    match open {
        Ok(Ok((stream, hs))) => {
            let connected = DiscordStatus {
                connected: true,
                username: hs.username.clone(),
                user_id: hs.user_id.clone(),
            };
            set_status(app, connected.clone());
            log::info!("Discord IPC connected as {}", hs.username);

            // Hold the socket open; read returns 0/Err when Discord exits,
            // the pipe breaks, or a reconnect becomes necessary.
            let stream_result =
                tauri::async_runtime::spawn_blocking(move || hold_until_disconnect(stream)).await;

            if let Err(e) = &stream_result {
                log::debug!("discord watch ended with error: {e}");
            }
            log::warn!("Discord IPC connection dropped");
            set_status(app, DiscordStatus::disconnected());
            true
        }
        Ok(Err(e)) => {
            log::debug!("discord IPC connect failed: {e}");
            set_status(app, DiscordStatus::disconnected());
            false
        }
        Err(_join) => false,
    }
}

/// Blocking read loop on an established connection until Discord goes away.
fn hold_until_disconnect(mut stream: Box<dyn ipc::ReadWrite>) -> std::io::Result<()> {
    let mut buf = [0u8; 4096];
    loop {
        match stream.read(&mut buf) {
            Ok(0) => return Ok(()),
            Err(e) => return Err(e),
            Ok(_) => {}
        }
    }
}

fn set_status(app: &AppHandle, status: DiscordStatus) {
    let state = app.state::<AppState>();
    *state.write_discord() = status.clone();
    let _ = app.emit(EVENT_DISCORD_STATUS, &status);
}
