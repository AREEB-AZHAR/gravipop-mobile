use macroquad::audio::{load_sound_from_bytes, play_sound_once, Sound};

// Pre-rendered static WAV files created and saved in assets/audio/
const CHIME_0: &[u8] = include_bytes!("../../assets/audio/chime_0.wav");
const CHIME_1: &[u8] = include_bytes!("../../assets/audio/chime_1.wav");
const CHIME_2: &[u8] = include_bytes!("../../assets/audio/chime_2.wav");
const CHIME_3: &[u8] = include_bytes!("../../assets/audio/chime_3.wav");
const CHIME_4: &[u8] = include_bytes!("../../assets/audio/chime_4.wav");
const CHIME_5: &[u8] = include_bytes!("../../assets/audio/chime_5.wav");
const CHIME_6: &[u8] = include_bytes!("../../assets/audio/chime_6.wav");
const CHIME_7: &[u8] = include_bytes!("../../assets/audio/chime_7.wav");
const SLINGSHOT: &[u8] = include_bytes!("../../assets/audio/slingshot.wav");
const GAME_OVER: &[u8] = include_bytes!("../../assets/audio/game_over.wav");
const CLICK: &[u8] = include_bytes!("../../assets/audio/click.wav");

pub struct AudioEngine {
    pub sound_enabled: bool,
    chime_sounds: Vec<Sound>,
    sling_sound: Option<Sound>,
    game_over_sound: Option<Sound>,
    click_sound: Option<Sound>,
}

impl AudioEngine {
    pub async fn new() -> Self {
        let mut chime_sounds = Vec::with_capacity(8);
        let chime_slices = [
            CHIME_0, CHIME_1, CHIME_2, CHIME_3,
            CHIME_4, CHIME_5, CHIME_6, CHIME_7,
        ];
        for bytes in chime_slices {
            if let Ok(snd) = load_sound_from_bytes(bytes).await {
                chime_sounds.push(snd);
            }
        }

        let sling_sound = load_sound_from_bytes(SLINGSHOT).await.ok();
        let game_over_sound = load_sound_from_bytes(GAME_OVER).await.ok();
        let click_sound = load_sound_from_bytes(CLICK).await.ok();

        Self {
            sound_enabled: true,
            chime_sounds,
            sling_sound,
            game_over_sound,
            click_sound,
        }
    }

    pub fn play_fusion_chime(&self, combo_count: usize) {
        if !self.sound_enabled || self.chime_sounds.is_empty() {
            return;
        }
        let index = (combo_count.saturating_sub(1)) % self.chime_sounds.len();
        play_sound_once(&self.chime_sounds[index]);
    }

    pub fn play_slingshot(&self) {
        if !self.sound_enabled {
            return;
        }
        if let Some(ref snd) = self.sling_sound {
            play_sound_once(snd);
        }
    }

    pub fn play_game_over(&self) {
        if !self.sound_enabled {
            return;
        }
        if let Some(ref snd) = self.game_over_sound {
            play_sound_once(snd);
        }
    }

    pub fn play_victory(&self) {
        if !self.sound_enabled || self.chime_sounds.is_empty() {
            return;
        }
        let top_idx = self.chime_sounds.len() - 1;
        play_sound_once(&self.chime_sounds[top_idx]);
    }

    pub fn play_click(&self) {
        if !self.sound_enabled {
            return;
        }
        if let Some(ref snd) = self.click_sound {
            play_sound_once(snd);
        }
    }

    pub fn set_sound_enabled(&mut self, enabled: bool) {
        self.sound_enabled = enabled;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedded_wav_files_are_valid() {
        let all_files = [
            CHIME_0, CHIME_1, CHIME_2, CHIME_3,
            CHIME_4, CHIME_5, CHIME_6, CHIME_7,
            SLINGSHOT, GAME_OVER, CLICK,
        ];
        for file in all_files {
            assert!(file.len() > 44, "Audio file is too small");
            assert_eq!(&file[0..4], b"RIFF", "Missing RIFF header");
            assert_eq!(&file[8..12], b"WAVE", "Missing WAVE identifier");
            assert_eq!(&file[12..16], b"fmt ", "Missing fmt chunk");
        }
    }
}
