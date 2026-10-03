use macroquad::audio::{load_sound_from_bytes, play_sound_once, Sound};

pub struct AudioEngine {
    pub sound_enabled: bool,
    chime_sounds: Vec<Sound>,
    sling_sound: Option<Sound>,
    game_over_sound: Option<Sound>,
}

impl AudioEngine {
    pub async fn new() -> Self {
        let mut chime_sounds = Vec::new();
        // Frequencies for pentatonic scale: C5, D5, E5, G5, A5, C6, D6, E6
        let freqs = [523.25, 587.33, 659.25, 783.99, 880.00, 1046.50, 1174.66, 1318.51];
        for &freq in &freqs {
            let wav_bytes = generate_bell_chime_wav(freq, 0.45);
            if let Ok(snd) = load_sound_from_bytes(&wav_bytes).await {
                chime_sounds.push(snd);
            }
        }

        let sling_wav = generate_woosh_wav(0.20);
        let sling_sound = load_sound_from_bytes(&sling_wav).await.ok();

        let over_wav = generate_low_boom_wav(0.6);
        let game_over_sound = load_sound_from_bytes(&over_wav).await.ok();

        Self {
            sound_enabled: true,
            chime_sounds,
            sling_sound,
            game_over_sound,
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
}

/// Generates a valid in-memory PCM 16-bit 44100Hz mono WAV buffer of a harmonic bell chime
fn generate_bell_chime_wav(freq: f32, duration_secs: f32) -> Vec<u8> {
    let sample_rate = 44100u32;
    let total_samples = (sample_rate as f32 * duration_secs) as usize;
    let mut samples: Vec<i16> = Vec::with_capacity(total_samples);

    for i in 0..total_samples {
        let t = i as f32 / sample_rate as f32;
        // Exponential bell decay envelope
        let env = (-t * 8.0).exp();
        // Fundamental + 1st overtone harmonic
        let fundamental = (t * freq * 2.0 * std::f32::consts::PI).sin();
        let overtone = (t * freq * 2.76 * 2.0 * std::f32::consts::PI).sin() * 0.35;
        let mixed = (fundamental + overtone) * env * 0.7;
        let sample_i16 = (mixed.clamp(-1.0, 1.0) * 32767.0) as i16;
        samples.push(sample_i16);
    }

    create_wav_container(&samples, sample_rate)
}

fn generate_woosh_wav(duration_secs: f32) -> Vec<u8> {
    let sample_rate = 44100u32;
    let total_samples = (sample_rate as f32 * duration_secs) as usize;
    let mut samples: Vec<i16> = Vec::with_capacity(total_samples);

    for i in 0..total_samples {
        let t = i as f32 / sample_rate as f32;
        let env = (-(t - 0.05).powi(2) * 120.0).exp();
        let pitch = 220.0 + t * 450.0;
        let s = (t * pitch * 2.0 * std::f32::consts::PI).sin() * env * 0.5;
        samples.push((s.clamp(-1.0, 1.0) * 32767.0) as i16);
    }

    create_wav_container(&samples, sample_rate)
}

fn generate_low_boom_wav(duration_secs: f32) -> Vec<u8> {
    let sample_rate = 44100u32;
    let total_samples = (sample_rate as f32 * duration_secs) as usize;
    let mut samples: Vec<i16> = Vec::with_capacity(total_samples);

    for i in 0..total_samples {
        let t = i as f32 / sample_rate as f32;
        let env = (-t * 5.5).exp();
        let pitch = (140.0 - t * 90.0).max(35.0);
        let s = (t * pitch * 2.0 * std::f32::consts::PI).sin() * env * 0.8;
        samples.push((s.clamp(-1.0, 1.0) * 32767.0) as i16);
    }

    create_wav_container(&samples, sample_rate)
}

fn create_wav_container(samples: &[i16], sample_rate: u32) -> Vec<u8> {
    let num_channels = 1u16;
    let bits_per_sample = 16u16;
    let byte_rate = sample_rate * num_channels as u32 * (bits_per_sample as u32 / 8);
    let block_align = num_channels * (bits_per_sample / 8);
    let data_len = (samples.len() * 2) as u32;
    let riff_chunk_size = 36 + data_len;

    let mut wav = Vec::with_capacity(44 + samples.len() * 2);
    // RIFF Header
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&riff_chunk_size.to_le_bytes());
    wav.extend_from_slice(b"WAVE");
    // "fmt " Subchunk
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes()); // Subchunk1Size (16 for PCM)
    wav.extend_from_slice(&1u16.to_le_bytes());  // AudioFormat (1 for PCM)
    wav.extend_from_slice(&num_channels.to_le_bytes());
    wav.extend_from_slice(&sample_rate.to_le_bytes());
    wav.extend_from_slice(&byte_rate.to_le_bytes());
    wav.extend_from_slice(&block_align.to_le_bytes());
    wav.extend_from_slice(&bits_per_sample.to_le_bytes());
    // "data" Subchunk
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_len.to_le_bytes());

    for &sample in samples {
        wav.extend_from_slice(&sample.to_le_bytes());
    }

    wav
}
