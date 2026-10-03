use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveData {
    pub high_score: u64,
    pub stardust: u64,
    pub ads_removed: bool,
    pub celestial_pass_unlocked: bool,
    pub equipped_skin: String,
    pub unlocked_skins: Vec<String>,
    pub runs_played: u32,
    pub sound_enabled: bool,
    pub haptics_enabled: bool,
}

impl Default for SaveData {
    fn default() -> Self {
        Self {
            high_score: 0,
            stardust: 100, // Starter bonus
            ads_removed: false,
            celestial_pass_unlocked: false,
            equipped_skin: "Cosmic Neon".to_string(),
            unlocked_skins: vec!["Cosmic Neon".to_string()],
            runs_played: 0,
            sound_enabled: true,
            haptics_enabled: true,
        }
    }
}

pub struct SaveManager {
    save_path: PathBuf,
}

impl SaveManager {
    pub fn new() -> Self {
        let save_path = PathBuf::from("gravipop_save.json");
        Self { save_path }
    }

    pub fn load(&self) -> SaveData {
        if let Ok(content) = fs::read_to_string(&self.save_path) {
            if let Ok(data) = serde_json::from_str::<SaveData>(&content) {
                return data;
            }
        }
        SaveData::default()
    }

    pub fn save(&self, data: &SaveData) -> Result<(), std::io::Error> {
        let json = serde_json::to_string_pretty(data)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        fs::write(&self.save_path, json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_data_default() {
        let data = SaveData::default();
        assert_eq!(data.high_score, 0);
        assert_eq!(data.stardust, 100);
        assert!(!data.ads_removed);
        assert!(data.unlocked_skins.contains(&"Cosmic Neon".to_string()));
    }

    #[test]
    fn test_save_data_serialization() {
        let mut data = SaveData::default();
        data.high_score = 45000;
        data.stardust = 1200;
        data.ads_removed = true;

        let json = serde_json::to_string(&data).expect("Serialization failed");
        let deserialized: SaveData = serde_json::from_str(&json).expect("Deserialization failed");

        assert_eq!(deserialized.high_score, 45000);
        assert_eq!(deserialized.stardust, 1200);
        assert!(deserialized.ads_removed);
    }
}
