// Virtual canvas
pub const VIRTUAL_WIDTH: f32 = 720.0;
pub const VIRTUAL_HEIGHT: f32 = 1280.0;

// ── Container ("the jar") ─────────────────────────────────────────────────────
pub const JAR_LEFT: f32 = 90.0;
pub const JAR_RIGHT: f32 = 630.0;
pub const JAR_TOP_LINE: f32 = 340.0; // open rim / overflow threshold y
pub const JAR_BOTTOM: f32 = 1110.0; // floor of the jar
pub const JAR_WIDTH: f32 = JAR_RIGHT - JAR_LEFT; // 540.0
pub const JAR_HEIGHT: f32 = JAR_BOTTOM - JAR_TOP_LINE; // 750.0

// ── Drop zone ─────────────────────────────────────────────────────────────────
pub const DROP_Y: f32 = 245.0; // y where new bodies appear before dropping
pub const DROP_COOLDOWN: f32 = 0.38; // seconds between drops

// ── Physics ───────────────────────────────────────────────────────────────────
pub const GRAVITY_ACCEL: f32 = 980.0;
pub const WALL_RESTITUTION: f32 = 0.24;
pub const FLOOR_RESTITUTION: f32 = 0.26;
pub const FLOOR_FRICTION: f32 = 0.86;
pub const BODY_RESTITUTION: f32 = 0.20;
pub const MERGE_COOLDOWN: f32 = 0.25;
pub const COLLISION_PASSES: usize = 6;

// ── Danger / game-over ────────────────────────────────────────────────────────
pub const DANGER_TIME: f32 = 5.0;
pub const CRITICAL_TIME_LIMIT: f32 = DANGER_TIME; // alias used in hud

// ── Abilities (Powerups) ──────────────────────────────────────────────────────
pub const MAX_ABILITY_CHARGES: u32 = 3;
pub const ABILITY_EARN_TIER: usize = 3; // GasGiant or higher grants a charge
pub const MAX_SUPER_FLARE_CHARGES: u32 = 2; // Super Solar Flare stores up to 2 usages
pub const SUPER_FLARE_SCORE_INTERVAL: u64 = 50_000; // Earn 1 charge every 50k points
pub const GRAVITY_WAVE_GRACE_DURATION: f32 = 3.5; // Seconds of overflow immunity when Gravity Wave is triggered
pub const OVERFLOW_ACTIVE_DROP_GRACE: f32 = 0.05; // Transit grace when dropping during active overflow

// ── Display & Resolution Profiles ─────────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub enum ResolutionProfile {
    LowBattery, // 540x960
    Standard,   // 720x1280
    #[default]
    HighDef,    // 1080x1920
    Ultra,      // 1440x2560
    Extreme4K,  // 2160x3840
}

impl ResolutionProfile {
    pub const ALL: [ResolutionProfile; 5] = [
        ResolutionProfile::LowBattery,
        ResolutionProfile::Standard,
        ResolutionProfile::HighDef,
        ResolutionProfile::Ultra,
        ResolutionProfile::Extreme4K,
    ];

    pub fn dimensions(self) -> (u32, u32) {
        match self {
            Self::LowBattery => (540, 960),
            Self::Standard => (720, 1280),
            Self::HighDef => (1080, 1920),
            Self::Ultra => (1440, 2560),
            Self::Extreme4K => (2160, 3840),
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::LowBattery => "540 x 960 (Power Saver)",
            Self::Standard => "720 x 1280 (Standard HD)",
            Self::HighDef => "1080 x 1920 (Full HD 1080p)",
            Self::Ultra => "1440 x 2560 (Ultra QHD 1440p)",
            Self::Extreme4K => "2160 x 3840 (4K UHD Extreme)",
        }
    }

    pub fn recommendation(self) -> &'static str {
        match self {
            Self::LowBattery => "Recommended for Budget & Older Devices (Max FPS)",
            Self::Standard => "Recommended for Balanced Everyday Mobile Play",
            Self::HighDef => "Recommended Baseline for 1080p Screens, Laptops & Phones",
            Self::Ultra => "Recommended for 1440p High-DPI Displays & Tablets",
            Self::Extreme4K => "Recommended for 4K UHD Monitors & OLED TVs (Maximum Sharpness)",
        }
    }
}

// ── Economy / Ads ─────────────────────────────────────────────────────────────
pub const STARDUST_PER_FUSION_BASE: u32 = 5;
pub const REMOVE_ADS_PRODUCT_ID: &str = "com.gravipop.removeads";
pub const CELESTIAL_PASS_PRODUCT_ID: &str = "com.gravipop.starpass";
/// Infrequent post-session ad cadence
pub const INTERSTITIAL_RUN_INTERVAL: u32 = 5;
pub const FREE_REVIVES_PER_RUN: u32 = 1;

