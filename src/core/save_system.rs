use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Versioned player save data. Old fields are kept with `#[serde(default)]`
/// so saves from earlier builds load safely.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveData {
    // ── Core ────────────────────────────────────────────────────────────────
    pub high_score: u64,
    pub stardust: u64,
    pub runs_played: u32,

    // ── Sector progression (one star-count per sector index) ─────────────────
    /// Stars earned per sector (0 = not completed, 1–3 = done).
    #[serde(default)]
    pub sector_stars: Vec<u8>,
    /// Index of the highest sector that has been unlocked.
    #[serde(default)]
    pub sectors_unlocked: usize,

    // ── Monetization / entitlements ──────────────────────────────────────────
    pub ads_removed: bool,
    pub celestial_pass_unlocked: bool,

    // ── Cosmetics ───────────────────────────────────────────────────────────
    pub equipped_skin: String,
    pub unlocked_skins: Vec<String>,

    // ── Settings ────────────────────────────────────────────────────────────
    #[serde(default = "default_true")]
    pub sound_enabled: bool,
    #[serde(default = "default_true")]
    pub haptics_enabled: bool,
}

fn default_true() -> bool { true }

impl Default for SaveData {
    fn default() -> Self {
        Self {
            high_score: 0,
            stardust: 100,       // starter gift
            runs_played: 0,
            sector_stars: vec![0u8; 16],
            sectors_unlocked: 0, // first sector always unlocked
            ads_removed: false,
            celestial_pass_unlocked: false,
            equipped_skin: "Cosmic Neon".to_string(),
            unlocked_skins: vec!["Cosmic Neon".to_string()],
            sound_enabled: true,
            haptics_enabled: true,
        }
    }
}

impl SaveData {
    /// Marks a sector complete (up to 3 stars). Returns true if this is a new
    /// personal best for that sector.
    pub fn complete_sector(&mut self, idx: usize, stars: u8) -> bool {
        if self.sector_stars.len() <= idx {
            self.sector_stars.resize(idx + 1, 0);
        }
        let prev = self.sector_stars[idx];
        if stars > prev {
            self.sector_stars[idx] = stars;
        }
        // Unlock the next sector
        if self.sectors_unlocked <= idx {
            self.sectors_unlocked = idx + 1;
        }
        stars > prev
    }

    pub fn stars_for(&self, idx: usize) -> u8 {
        self.sector_stars.get(idx).copied().unwrap_or(0)
    }

    pub fn is_sector_unlocked(&self, idx: usize) -> bool {
        idx == 0 || idx <= self.sectors_unlocked
    }
}

pub struct SaveManager {
    save_path: PathBuf,
}

impl Default for SaveManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SaveManager {
    pub fn new() -> Self {
        Self { save_path: PathBuf::from("gravipop_save.json") }
    }

    pub fn load(&self) -> SaveData {
        if let Ok(text) = fs::read_to_string(&self.save_path) {
            if let Ok(data) = serde_json::from_str::<SaveData>(&text) {
                return data;
            }
        }
        SaveData::default()
    }

    pub fn save(&self, data: &SaveData) -> Result<(), std::io::Error> {
        let json = serde_json::to_string_pretty(data)
            .map_err(std::io::Error::other)?;
        fs::write(&self.save_path, json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sector_progress() {
        let mut data = SaveData::default();
        assert_eq!(data.stars_for(0), 0);
        assert!(data.is_sector_unlocked(0));
        assert!(!data.is_sector_unlocked(1));

        data.complete_sector(0, 2);
        assert_eq!(data.stars_for(0), 2);
        assert!(data.is_sector_unlocked(1));

        // Higher star replaces lower
        data.complete_sector(0, 3);
        assert_eq!(data.stars_for(0), 3);

        // Lower star does NOT replace higher
        data.complete_sector(0, 1);
        assert_eq!(data.stars_for(0), 3);
    }

    #[test]
    fn test_save_roundtrip() {
        let data = SaveData {
            high_score: 9999,
            stardust: 500,
            sector_stars: vec![3, 2, 1, 0],
            sectors_unlocked: 3,
            ..Default::default()
        };

        let json = serde_json::to_string(&data).unwrap();
        let loaded: SaveData = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.high_score, 9999);
        assert_eq!(loaded.sectors_unlocked, 3);
        assert_eq!(loaded.stars_for(1), 2);
    }
}
