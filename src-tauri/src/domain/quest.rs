//! Quest aggregate — the typed replacement for the stringly-typed
//! `DiscordQuest` wire struct (which stays on the boundary in Phase 0).

use std::fmt;

use super::reward::Reward;
use super::target::LaunchTarget;

/// Branded quest identifier. `QuestId("endfield_1")` — distinct from
/// `client_id` at the type level so the two can never be mixed up.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct QuestId(pub String);

impl QuestId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for QuestId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<&str> for QuestId {
    fn from(s: &str) -> Self {
        QuestId(s.to_string())
    }
}

impl From<String> for QuestId {
    fn from(s: String) -> Self {
        QuestId(s)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Quest {
    pub id: QuestId,
    pub title: String,
    pub game_name: String,
    pub target: LaunchTarget,
    /// Discord application client_id this quest attaches to.
    pub client_id: String,
    pub reward: Reward,
    /// Saved progress 0..=100, clamped on construction.
    pub saved_percent: u8,
}

impl Quest {
    pub fn new(
        id: impl Into<QuestId>,
        title: impl Into<String>,
        game_name: impl Into<String>,
        target: LaunchTarget,
        client_id: impl Into<String>,
        reward: Reward,
        saved_percent: u8,
    ) -> Self {
        Quest {
            id: id.into(),
            title: title.into(),
            game_name: game_name.into(),
            target,
            client_id: client_id.into(),
            reward,
            saved_percent: saved_percent.min(100),
        }
    }

    /// Case-insensitive match across title, game name and target label.
    pub fn matches(&self, query: &str) -> bool {
        let q = query.to_lowercase();
        self.title.to_lowercase().contains(&q)
            || self.game_name.to_lowercase().contains(&q)
            || self.target.label().to_lowercase().contains(&q)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Quest {
        Quest::new(
            "endfield_1",
            "Companionship Celebration",
            "Arknights: Endfield",
            LaunchTarget::Exe {
                exe_name: "Endfield.exe".into(),
            },
            "1241071192534597652",
            Reward::Orbs(700),
            79,
        )
    }

    #[test]
    fn clamps_saved_percent() {
        assert_eq!(
            Quest::new(
                "x",
                "t",
                "g",
                LaunchTarget::Console,
                "c",
                Reward::Orbs(1),
                150
            )
            .saved_percent,
            100
        );
    }

    #[test]
    fn matches_is_case_insensitive_across_fields() {
        let q = sample();
        assert!(q.matches("endfield"));
        assert!(q.matches("ENDFIELD"));
        assert!(q.matches("companionship"));
        assert!(q.matches("Endfield.exe"));
        assert!(!q.matches("genshin"));
    }

    #[test]
    fn quest_id_display() {
        assert_eq!(sample().id.to_string(), "endfield_1");
    }
}
