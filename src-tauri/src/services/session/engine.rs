//! Session engine implementation.

use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::watch;

use crate::app::error::AppError;
use crate::app::state::AppState;
use crate::domain::quest::Quest;
use crate::domain::session::{Session, SessionId, SessionKind};
use crate::domain::target::LaunchTarget;
use crate::infra::config::{GAME_QUEST_DURATION_SEC, VIDEO_QUEST_DURATION_SEC};
use crate::services::discord::activity::{self, ActivityGuard, ActivityRequest};
use crate::services::spoofer::orchestrator;

pub const EVENT_SESSION_STARTED: &str = "session://started";
pub const EVENT_SESSION_PROGRESS: &str = "session://progress";
pub const EVENT_SESSION_FINISHED: &str = "session://finished";
pub const EVENT_SESSION_STOPPED: &str = "session://stopped";

const TICK_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum StopReason {
    User,
    Error,
}

#[derive(Debug, Clone, Serialize)]
pub struct SessionStarted {
    pub session_id: String,
    pub quest_id: String,
    pub game_name: String,
    pub exe_name: String,
    pub target_sec: u64,
    pub initial_percent: u8,
}

#[derive(Debug, Clone, Serialize)]
pub struct SessionProgress {
    pub session_id: String,
    pub percent: u8,
    pub elapsed_sec: u64,
    pub remaining_sec: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SessionFinished {
    pub session_id: String,
    pub quest_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SessionStopped {
    pub session_id: String,
    pub reason: StopReason,
    /// User-safe reason for a failure stop (None for user stops or finished
    /// sessions). Surfaced verbatim in the UI error banner.
    pub message: Option<String>,
}

/// Handle to the running session task, kept in `AppState` so `stop_session`
/// can signal a clean stop and abort as a fallback.
pub struct SessionTask {
    stop: watch::Sender<Option<StopReason>>,
    pub handle: tauri::async_runtime::JoinHandle<()>,
}

enum Outcome {
    Finished,
    Stopped(StopReason, Option<String>),
}

/// Start a quest session. Errors if a session is already running.
pub async fn start(app: &AppHandle, quest: Quest) -> Result<Session, AppError> {
    let state = app.state::<AppState>();
    if state.read_session().is_some() {
        return Err(AppError::SessionActive);
    }

    let session = Session {
        id: SessionId(format!("session_{}", quest.id.as_str())),
        quest: quest.clone(),
        kind: session_kind(&quest.target),
        started_at: Instant::now(),
        target_sec: quest_duration(&quest),
        initial_percent: quest.saved_percent,
    };

    let (stop_tx, stop_rx) = watch::channel(None);
    *state.write_session() = Some(session.clone());

    let handle = tauri::async_runtime::spawn(run(app.clone(), session.clone(), stop_rx));
    *state.write_session_task() = Some(SessionTask {
        stop: stop_tx,
        handle,
    });

    let _ = app.emit(
        EVENT_SESSION_STARTED,
        SessionStarted {
            session_id: session.id.to_string(),
            quest_id: session.quest.id.to_string(),
            game_name: session.quest.game_name.clone(),
            exe_name: session.quest.target.wire_exe_name(),
            target_sec: session.target_sec.as_secs(),
            initial_percent: session.initial_percent,
        },
    );
    log::info!("session started: {}", session.id);
    Ok(session)
}

/// Stop the running session (idempotent when nothing is running).
pub async fn stop(app: &AppHandle) -> Result<(), AppError> {
    let state = app.state::<AppState>();
    let task = state.write_session_task().take();
    let session = state.write_session().take();

    if let Some(task) = task {
        let _ = task.stop.send(Some(StopReason::User));
        task.handle.abort();
    }

    cleanup(app).await;

    if let Some(session) = session {
        let _ = app.emit(
            EVENT_SESSION_STOPPED,
            SessionStopped {
                session_id: session.id.to_string(),
                reason: StopReason::User,
                message: None,
            },
        );
        log::info!("session stopped: {}", session.id);
    }
    Ok(())
}

/// Whether a session is currently running, plus the running session's wire
/// view for re-hydrating the UI after a reload.
pub fn status(app: &AppHandle) -> Option<SessionStarted> {
    let state = app.state::<AppState>();
    let session = state.read_session();
    session.as_ref().map(|s| SessionStarted {
        session_id: s.id.to_string(),
        quest_id: s.quest.id.to_string(),
        game_name: s.quest.game_name.clone(),
        exe_name: s.quest.target.wire_exe_name(),
        target_sec: s.target_sec.as_secs(),
        initial_percent: s.initial_percent,
    })
}

async fn run(app: AppHandle, session: Session, mut stop_rx: watch::Receiver<Option<StopReason>>) {
    let activity = match launch(&app, &session).await {
        Ok(guard) => guard,
        Err(e) => {
            log::warn!("session {} launch failed: {}", session.id, e.log_detail());
            finish(
                &app,
                &session,
                Outcome::Stopped(StopReason::Error, Some(e.message())),
            )
            .await;
            return;
        }
    };
    {
        let state = app.state::<AppState>();
        *state.write_activity() = Some(activity);
    }

    let mut tick = tokio::time::interval(TICK_INTERVAL);
    tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        tokio::select! {
            _ = tick.tick() => {
                let now = Instant::now();
                let _ = app.emit(EVENT_SESSION_PROGRESS, SessionProgress {
                    session_id: session.id.to_string(),
                    percent: session.progress(now),
                    elapsed_sec: session.elapsed_sec(now),
                    remaining_sec: session.remaining_sec(now),
                });
                if session.is_finished(now) {
                    finish(&app, &session, Outcome::Finished).await;
                    return;
                }
            }
            result = stop_rx.changed() => {
                if result.is_ok() {
                    let reason: Option<StopReason> = *stop_rx.borrow_and_update();
                    if let Some(reason) = reason {
                        finish(&app, &session, Outcome::Stopped(reason, None)).await;
                    }
                }
                return;
            }
        }
    }
}

/// Launch the simulation matching the session kind.
async fn launch(app: &AppHandle, session: &Session) -> Result<ActivityGuard, AppError> {
    #[cfg(not(target_os = "windows"))]
    let _ = app;
    let quest = &session.quest;
    let duration_secs = session.target_sec.as_secs();

    match &session.kind {
        SessionKind::Exe => {
            // The process spoofer is Windows-only: Discord's game detection
            // matches win32 executables and no such detection exists on
            // Linux/macOS. On those platforms we fall back to activity-only
            // simulation — setting the game's own Rich Presence over IPC so
            // Discord still reports "Playing <game>". Whether a given quest's
            // backend credits Rich Presence for progress is game/quest-specific.
            #[cfg(target_os = "windows")]
            {
                let app = app.clone();
                let game_name = quest.game_name.clone();
                let exe_names = orchestrator::exe_names_for_simulation(
                    app.state::<AppState>().read_catalog().as_ref(),
                    &quest.game_name,
                );
                tauri::async_runtime::spawn_blocking(move || {
                    orchestrator::spawn_exe_simulation(&app, &exe_names, &game_name)
                })
                .await
                .map_err(|e| AppError::Internal(format!("spoofer task panicked: {e}")))??;
            }

            hold_ipc_activity(
                &quest.client_id,
                &format!("Completing Quest: {}", quest.title),
                &format!("Earning {}", quest.reward.to_display()),
                duration_secs,
            )
            .await
        }
        SessionKind::Console => {
            hold_ipc_activity(
                &quest.client_id,
                "Playing on PlayStation 5 / Xbox",
                "Completing Console Quest",
                duration_secs,
            )
            .await
        }
        SessionKind::Stream => {
            hold_ipc_activity(
                &quest.client_id,
                "Streaming Game to Channel",
                "Completing Stream Quest",
                duration_secs,
            )
            .await
        }
    }
}

/// Open and hold a Rich Presence connection with a `[start, end]` window
/// (I/O in `spawn_blocking` so the engine task never blocks).
async fn hold_ipc_activity(
    client_id: &str,
    details: &str,
    state: &str,
    duration_secs: u64,
) -> Result<ActivityGuard, AppError> {
    let req = ActivityRequest {
        client_id: client_id.to_string(),
        details: details.to_string(),
        state: state.to_string(),
        duration_secs,
    };
    tauri::async_runtime::spawn_blocking(move || activity::hold_activity(req))
        .await
        .map_err(|e| AppError::Internal(format!("IPC activity task panicked: {e}")))?
        .map_err(AppError::Internal)
}

/// Kill spoofer PIDs + clear IPC activity, then clear the running session
/// from `AppState` and emit the terminal event.
async fn finish(app: &AppHandle, session: &Session, outcome: Outcome) {
    cleanup(app).await;

    let state = app.state::<AppState>();
    *state.write_session() = None;
    *state.write_session_task() = None;

    match outcome {
        Outcome::Finished => {
            let _ = app.emit(
                EVENT_SESSION_FINISHED,
                SessionFinished {
                    session_id: session.id.to_string(),
                    quest_id: session.quest.id.to_string(),
                },
            );
            log::info!("session finished: {}", session.id);
        }
        Outcome::Stopped(reason, message) => {
            let _ = app.emit(
                EVENT_SESSION_STOPPED,
                SessionStopped {
                    session_id: session.id.to_string(),
                    reason,
                    message,
                },
            );
            log::info!("session stopped ({reason:?}): {}", session.id);
        }
    }
}

/// Stop spoofer processes (Windows), stop the held activity (clearing the
/// presence + closing the socket), so no ghost presence outlives the session.
async fn cleanup(app: &AppHandle) {
    let app = app.clone();
    let _ = tauri::async_runtime::spawn_blocking(move || {
        orchestrator::stop_all(&app);
        if let Some(mut guard) = app.state::<AppState>().write_activity().take() {
            guard.stop();
        }
        let _ = crate::services::discord::activity::clear_activity();
    })
    .await;
}

fn session_kind(target: &LaunchTarget) -> SessionKind {
    match target {
        LaunchTarget::Exe { .. } => SessionKind::Exe,
        LaunchTarget::Console => SessionKind::Console,
        LaunchTarget::Stream => SessionKind::Stream,
    }
}

/// Target duration: 30s for video quests, 15 minutes otherwise. Matches the
/// legacy frontend heuristic, which checked the wire `exe_name` marker for
/// "video"; the title is checked too for quests that only carry it there.
fn quest_duration(quest: &Quest) -> Duration {
    let hay = format!("{} {}", quest.title, quest.target.wire_exe_name()).to_lowercase();
    if hay.contains("video") {
        Duration::from_secs(VIDEO_QUEST_DURATION_SEC)
    } else {
        Duration::from_secs(GAME_QUEST_DURATION_SEC)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duration_is_30s_for_video_quests() {
        let quest = Quest::new(
            "video_1",
            "Watch 2 videos",
            "YouTube",
            LaunchTarget::Stream,
            "1",
            crate::domain::reward::Reward::Orbs(10),
            0,
        );
        assert_eq!(quest_duration(&quest), Duration::from_secs(30));
    }

    #[test]
    fn duration_is_15m_for_games() {
        let quest = Quest::new(
            "endfield_1",
            "Companionship Celebration",
            "Arknights: Endfield",
            LaunchTarget::Exe {
                exe_name: "Endfield.exe".into(),
            },
            "2",
            crate::domain::reward::Reward::Orbs(700),
            0,
        );
        assert_eq!(quest_duration(&quest), Duration::from_secs(15 * 60));
    }

    #[test]
    fn session_kind_maps_targets() {
        assert_eq!(
            session_kind(&LaunchTarget::Exe {
                exe_name: "Endfield.exe".into()
            }),
            SessionKind::Exe
        );
        assert_eq!(session_kind(&LaunchTarget::Console), SessionKind::Console);
        assert_eq!(session_kind(&LaunchTarget::Stream), SessionKind::Stream);
    }
}
