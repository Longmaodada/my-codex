use std::{path::PathBuf, time::{Duration, SystemTime}};

use walkdir::WalkDir;

pub struct ActivityDetector {
    sessions_root: Option<PathBuf>,
}

impl ActivityDetector {
    pub fn discover() -> Self {
        let sessions_root = std::env::var_os("CODEX_HOME")
            .map(PathBuf::from)
            .or_else(|| dirs::home_dir().map(|home| home.join(".codex")))
            .map(|home| home.join("sessions"));
        Self { sessions_root }
    }

    /// Uses only file modification metadata. It never opens a session, auth, or config file.
    pub fn is_recently_active(&self, within: Duration) -> bool {
        let Some(root) = self.sessions_root.as_ref().filter(|root| root.is_dir()) else {
            return false;
        };
        let now = SystemTime::now();
        WalkDir::new(root)
            .follow_links(false)
            .max_depth(6)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| {
                entry.file_type().is_file()
                    && entry.path().extension().and_then(|value| value.to_str()) == Some("jsonl")
            })
            .filter_map(|entry| entry.metadata().ok()?.modified().ok())
            .any(|modified| now.duration_since(modified).map(|age| age <= within).unwrap_or(true))
    }
}

