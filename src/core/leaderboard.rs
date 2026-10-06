use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct LeaderboardEntry {
    pub display_name: String,
    pub high_score: u64,
}

pub struct LeaderboardClient {
    pub entries: Vec<LeaderboardEntry>,
    pub status: String,
    #[cfg(not(target_arch = "wasm32"))]
    native: super::leaderboard_native::Worker,
}

impl Default for LeaderboardClient {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            status: String::new(),
            #[cfg(not(target_arch = "wasm32"))]
            native: super::leaderboard_native::Worker::default(),
        }
    }
}

impl LeaderboardClient {
    pub fn configured() -> bool {
        true
    }

    pub async fn refresh(&mut self) {
        self.status = "Refreshing global leaderboard...".to_string();
        #[cfg(target_arch = "wasm32")]
        {
            crate::core::web_bridge::leaderboard_refresh();
            self.poll();
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.native.send(super::leaderboard_native::Request::Refresh);
            self.poll();
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
        #[cfg(not(target_arch = "wasm32"))]
        while let Ok(result) = self.native.results.try_recv() {
            match result {
                Ok((entries, status)) => { self.entries = entries; self.status = status; }
                Err(status) => self.status = status,
            }
        }
    }

    pub async fn submit(&mut self, display_name: &str, score: u64) {
        self.status = "Submitting score to global leaderboard...".to_string();
        #[cfg(target_arch = "wasm32")]
        {
            crate::core::web_bridge::leaderboard_submit(display_name, score);
            self.poll();
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.native.send(super::leaderboard_native::Request::Submit {
                display_name: display_name.to_string(), high_score: score,
            });
            self.poll();
        }
    }
}
