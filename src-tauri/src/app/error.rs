//! Typed application error contract shared by every Tauri command.
//!
//! Replaces the previous `Result<_, String>` soup with a single error
//! enum. Commands serialize as `{ "code": "...", "message": "..." }`;
//! the frontend renders `message` while `code` drives logic (e.g. disabling
//! the Start button on `SESSION_ACTIVE`).

use serde::ser::SerializeStruct;
use serde::Serialize;

/// Convenience alias for commands that return a typed value or `AppError`.
pub type AppResult<T> = Result<T, AppError>;

/// Machine-readable codes shared with the frontend contract (see §6.3).
pub const CODE_DISCORD_NOT_REACHABLE: &str = "DISCORD_NOT_REACHABLE";
pub const CODE_SESSION_ACTIVE: &str = "SESSION_ACTIVE";
pub const CODE_QUEST_NOT_FOUND: &str = "QUEST_NOT_FOUND";
pub const CODE_PLATFORM_UNSUPPORTED: &str = "PLATFORM_UNSUPPORTED";
pub const CODE_CATALOG_EMPTY: &str = "CATALOG_EMPTY";
pub const CODE_UPDATE_CHECK_FAILED: &str = "UPDATE_CHECK_FAILED";
pub const CODE_VALIDATION: &str = "VALIDATION";
pub const CODE_INTERNAL: &str = "INTERNAL";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppError {
    DiscordNotReachable,
    SessionActive,
    QuestNotFound,
    PlatformUnsupported,
    CatalogEmpty,
    /// Update-check failures (offline, non-200, bad payload). Detail is
    /// logged; the serialized message stays user-safe.
    UpdateCheckFailed(String),
    Validation(String),
    /// Internal failures. The detail string is logged but never surfaced to
    /// the frontend — the serialized `message` is always a safe generic.
    Internal(String),
}

impl AppError {
    pub fn code(&self) -> &'static str {
        match self {
            AppError::DiscordNotReachable => CODE_DISCORD_NOT_REACHABLE,
            AppError::SessionActive => CODE_SESSION_ACTIVE,
            AppError::QuestNotFound => CODE_QUEST_NOT_FOUND,
            AppError::PlatformUnsupported => CODE_PLATFORM_UNSUPPORTED,
            AppError::CatalogEmpty => CODE_CATALOG_EMPTY,
            AppError::UpdateCheckFailed(_) => CODE_UPDATE_CHECK_FAILED,
            AppError::Validation(_) => CODE_VALIDATION,
            AppError::Internal(_) => CODE_INTERNAL,
        }
    }

    /// User-safe message. Never includes internals; `Internal` detail only
    /// lives in the log.
    pub fn message(&self) -> String {
        match self {
            AppError::DiscordNotReachable => "Discord isn't running.".to_string(),
            AppError::SessionActive => "A session is already active.".to_string(),
            AppError::QuestNotFound => "Quest not found.".to_string(),
            AppError::PlatformUnsupported => {
                "This feature is only available on Windows.".to_string()
            }
            AppError::CatalogEmpty => "The game catalog is empty.".to_string(),
            AppError::UpdateCheckFailed(_) => "Couldn't check for updates.".to_string(),
            AppError::Validation(msg) => msg.clone(),
            AppError::Internal(_) => "Something went wrong. Please try again.".to_string(),
        }
    }

    /// Detail suitable for logging (internal errors carry their real cause).
    pub fn log_detail(&self) -> String {
        match self {
            AppError::Internal(detail) => format!("Internal error: {detail}"),
            AppError::UpdateCheckFailed(detail) => format!("Update check failed: {detail}"),
            other => other.message(),
        }
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code(), self.message())
    }
}

impl std::error::Error for AppError {}

impl Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut s = serializer.serialize_struct("AppError", 2)?;
        s.serialize_field("code", self.code())?;
        s.serialize_field("message", &self.message())?;
        s.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_match_contract() {
        assert_eq!(AppError::DiscordNotReachable.code(), "DISCORD_NOT_REACHABLE");
        assert_eq!(AppError::SessionActive.code(), "SESSION_ACTIVE");
        assert_eq!(AppError::QuestNotFound.code(), "QUEST_NOT_FOUND");
        assert_eq!(AppError::PlatformUnsupported.code(), "PLATFORM_UNSUPPORTED");
        assert_eq!(AppError::CatalogEmpty.code(), "CATALOG_EMPTY");
        assert_eq!(
            AppError::UpdateCheckFailed("offline".into()).code(),
            "UPDATE_CHECK_FAILED"
        );
        assert_eq!(
            AppError::Validation("x".into()).code(),
            "VALIDATION"
        );
        assert_eq!(AppError::Internal("boom".into()).code(), "INTERNAL");
    }

    #[test]
    fn serializes_to_contract_shape() {
        let json = serde_json::to_value(AppError::DiscordNotReachable).unwrap();
        assert_eq!(json["code"], "DISCORD_NOT_REACHABLE");
        assert_eq!(json["message"], "Discord isn't running.");
        assert_eq!(json.as_object().unwrap().len(), 2);
    }

    #[test]
    fn internal_message_never_leaks_detail() {
        let err = AppError::Internal("secret path /home/user".into());
        let json = serde_json::to_value(&err).unwrap();
        assert_eq!(json["message"], "Something went wrong. Please try again.");
        assert!(!json["message"].as_str().unwrap().contains("secret"));
    }

    #[test]
    fn update_check_failed_keeps_detail_out_of_message() {
        let err = AppError::UpdateCheckFailed("401 from api.github.com".into());
        let json = serde_json::to_value(&err).unwrap();
        assert_eq!(json["code"], "UPDATE_CHECK_FAILED");
        assert_eq!(json["message"], "Couldn't check for updates.");
        assert!(err.log_detail().contains("401"));
    }

    #[test]
    fn log_detail_keeps_internal_cause() {
        let err = AppError::Internal("boom".into());
        assert!(err.log_detail().contains("boom"));
    }

    #[test]
    fn validation_round_trips_message() {
        let json = serde_json::to_value(AppError::Validation("bad id".into())).unwrap();
        assert_eq!(json["code"], "VALIDATION");
        assert_eq!(json["message"], "bad id");
    }

    #[test]
    fn display_is_readable() {
        let err = AppError::SessionActive;
        assert_eq!(err.to_string(), "SESSION_ACTIVE: A session is already active.");
    }
}
