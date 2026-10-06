use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LeaderboardEntry {
    pub display_name: String,
    pub high_score: u64,
}

pub struct LeaderboardClient {
    pub entries: Vec<LeaderboardEntry>,
    pub status: String,
}

impl Default for LeaderboardClient {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            status: String::new(),
        }
    }
}

impl LeaderboardClient {
    pub fn configured() -> bool {
        crate::core::web_bridge::is_leaderboard_configured()
    }

    pub async fn refresh(&mut self) {
        #[cfg(target_arch = "wasm32")]
        {
            crate::core::web_bridge::leaderboard_refresh();
            self.poll();
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.status = "Online leaderboard is available in the web build".to_string();
        }
    }

    pub fn poll(&mut self) {
        #[cfg(target_arch = "wasm32")]
        {
            let status = crate::core::web_bridge::leaderboard_status();
            if !status.is_empty() {
                self.status = status;
            }
            if let Some(json_str) = crate::core::web_bridge::leaderboard_entries_json() {
                if let Ok(entries) = serde_json::from_str::<Vec<LeaderboardEntry>>(&json_str) {
                    self.entries = entries;
                }
            }
        }
    }

    pub async fn submit(&mut self, display_name: &str, score: u64) {
        #[cfg(target_arch = "wasm32")]
        {
            crate::core::web_bridge::leaderboard_submit(display_name, score);
            self.poll();
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = (display_name, score);
        }
    }
}
