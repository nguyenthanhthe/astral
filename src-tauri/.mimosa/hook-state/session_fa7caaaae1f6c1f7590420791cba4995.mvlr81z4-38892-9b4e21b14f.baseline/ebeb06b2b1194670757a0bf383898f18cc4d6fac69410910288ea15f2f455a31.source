//! Launch target for a quest — replaces the `"[Console Quest]"` /
//! `"[Stream Quest]"` / `"game.exe"` string markers.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LaunchTarget {
    /// A real game executable to spoof on disk.
    Exe { exe_name: String },
    /// Console quest (PS5 / Xbox) — Discord IPC activity, no process.
    Console,
    /// Voice-channel stream quest — Discord IPC activity, no process.
    Stream,
    /// Quest video (watched in the Discord client) — Discord IPC activity,
    /// no process.
    Video,
}

impl LaunchTarget {
    /// Human-readable label for the UI.
    pub fn label(&self) -> String {
        match self {
            LaunchTarget::Exe { exe_name } => exe_name.clone(),
            LaunchTarget::Console => "Console (PS5 / Xbox)".to_string(),
            LaunchTarget::Stream => "Voice stream".to_string(),
            LaunchTarget::Video => "Quest video".to_string(),
        }
    }

    /// Wire projection for the current frontend contract.
    ///
    /// Keeps the historical `[Console Quest]` / `[Stream Quest]` markers so the
    /// UI's `isNonExeQuest`/`questTargetLabel` keep working unchanged. The
    /// frontend migrates to typed targets in Phase 3 (T9); until then the
    /// markers only exist at this boundary, never in domain logic.
    pub fn wire_exe_name(&self) -> String {
        match self {
            LaunchTarget::Exe { exe_name } => exe_name.clone(),
            LaunchTarget::Console => "[Console Quest]".to_string(),
            LaunchTarget::Stream => "[Stream Quest]".to_string(),
            LaunchTarget::Video => "[Video Quest]".to_string(),
        }
    }

    pub fn is_exe(&self) -> bool {
        matches!(self, LaunchTarget::Exe { .. })
    }

    /// The executable name for `Exe` targets, `None` for the IPC-only kinds.
    pub fn exe_name(&self) -> Option<&str> {
        match self {
            LaunchTarget::Exe { exe_name } => Some(exe_name),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exe_label_is_the_executable() {
        let t = LaunchTarget::Exe {
            exe_name: "Endfield.exe".into(),
        };
        assert_eq!(t.label(), "Endfield.exe");
        assert!(t.is_exe());
    }

    #[test]
    fn console_stream_and_video_labels() {
        assert_eq!(LaunchTarget::Console.label(), "Console (PS5 / Xbox)");
        assert_eq!(LaunchTarget::Stream.label(), "Voice stream");
        assert_eq!(LaunchTarget::Video.label(), "Quest video");
        assert!(!LaunchTarget::Console.is_exe());
        assert!(!LaunchTarget::Stream.is_exe());
        assert!(!LaunchTarget::Video.is_exe());
    }

    #[test]
    fn wire_markers_preserve_fe_contract() {
        assert_eq!(LaunchTarget::Console.wire_exe_name(), "[Console Quest]");
        assert_eq!(LaunchTarget::Stream.wire_exe_name(), "[Stream Quest]");
        assert_eq!(LaunchTarget::Video.wire_exe_name(), "[Video Quest]");
        assert_eq!(
            LaunchTarget::Exe {
                exe_name: "Eve.exe".into()
            }
            .wire_exe_name(),
            "Eve.exe"
        );
    }
}
