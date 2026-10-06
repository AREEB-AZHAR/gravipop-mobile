use serde::{Deserialize, Serialize};
#[cfg(not(target_arch = "wasm32"))]
use std::fs;
#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

/// Versioned player save data. Old fields are kept with `#[serde(default)]`
/// so saves from earlier builds load safely.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveData {
    // ── Core ────────────────────────────────────────────────────────────────
    pub high_score: u64,
    pub stardust: u64,
    pub runs_played: u32,
    /// Public display name chosen at the end of a run.
    #[serde(default)]
    pub public_name: String,

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

fn default_true() -> bool {
    true
}

impl Default for SaveData {
    fn default() -> Self {
        Self {
            high_score: 0,
            stardust: 100, // starter gift
            runs_played: 0,
            public_name: String::new(),
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
    #[cfg(not(target_arch = "wasm32"))]
    save_path: PathBuf,
}

#[cfg(target_os = "android")]
fn android_save_path() -> PathBuf {
    use macroquad::miniquad::native::android::{self, ndk_sys::*};
    // The Java Activity supplies its app-private directory; Android does not
    // permit writing a relative save file into the process working directory.
    unsafe {
        let activity = android::ACTIVITY;
        if activity.is_null() { return PathBuf::from("gravipop_save.json"); }
        let env = android::attach_jni_env();
        let class = (**env).GetObjectClass.unwrap()(env, activity);
        let method = (**env).GetMethodID.unwrap()(env, class,
            b"getSaveDirectoryFromNative\0".as_ptr().cast(),
            b"()Ljava/lang/String;\0".as_ptr().cast());
        if method.is_null() {
            (**env).ExceptionClear.unwrap()(env);
            (**env).DeleteLocalRef.unwrap()(env, class);
            return PathBuf::from("gravipop_save.json");
        }
        let value = (**env).CallObjectMethodA.unwrap()(env, activity, method, std::ptr::null()) as jstring;
        (**env).DeleteLocalRef.unwrap()(env, class);
        if value.is_null() {
            (**env).ExceptionClear.unwrap()(env);
            return PathBuf::from("gravipop_save.json");
        }
        let chars = (**env).GetStringUTFChars.unwrap()(env, value, std::ptr::null_mut());
        let directory = if chars.is_null() { String::new() }
            else { std::ffi::CStr::from_ptr(chars).to_string_lossy().into_owned() };
        if !chars.is_null() { (**env).ReleaseStringUTFChars.unwrap()(env, value, chars); }
        (**env).DeleteLocalRef.unwrap()(env, value);
        PathBuf::from(directory).join("gravipop_save.json")
    }
}

impl Default for SaveManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SaveManager {
    pub fn new() -> Self {
        Self {
            #[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
            save_path: PathBuf::from("gravipop_save.json"),
            #[cfg(target_os = "android")]
            save_path: android_save_path(),
        }
    }

    pub fn load(&self) -> SaveData {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(text) = crate::core::web_bridge::storage_get("gravipop.save.v1") {
                if let Ok(data) = serde_json::from_str::<SaveData>(&text) {
                    return data;
                }
            }
            return SaveData::default();
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Ok(text) = fs::read_to_string(&self.save_path) {
                if let Ok(data) = serde_json::from_str::<SaveData>(&text) {
                    return data;
                }
            }
            SaveData::default()
        }
    }

    pub fn save(&self, data: &SaveData) -> Result<(), std::io::Error> {
        let json = serde_json::to_string_pretty(data).map_err(std::io::Error::other)?;
        #[cfg(target_arch = "wasm32")]
        {
            crate::core::web_bridge::storage_set("gravipop.save.v1", &json);
            Ok(())
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            fs::write(&self.save_path, json)
        }
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
