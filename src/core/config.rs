pub const VIRTUAL_WIDTH: f32 = 720.0;
pub const VIRTUAL_HEIGHT: f32 = 1280.0;

// Physics constants
pub const GRAVITATIONAL_CONSTANT: f32 = 18000.0;
pub const DAMPING: f32 = 0.998;
pub const RESTITUTION: f32 = 0.55;
pub const CORE_RADIUS: f32 = 38.0;
pub const EVENT_HORIZON_RADIUS: f32 = 310.0;
pub const CRITICAL_TIME_LIMIT: f32 = 4.0; // Seconds allowed outside safe orbit before collapse

// Gameplay constants
pub const MAX_SLING_SPEED: f32 = 480.0;
pub const SLING_SENSITIVITY: f32 = 2.4;
pub const COMBO_TIMEOUT_SECS: f32 = 2.2;
pub const FREE_REVIVES_PER_RUN: u32 = 1;
pub const INTERSTITIAL_RUN_INTERVAL: u32 = 3;

// Economy constants
pub const STARDUST_PER_FUSION_BASE: u32 = 5;
pub const REMOVE_ADS_PRODUCT_ID: &str = "com.gravipop.removeads";
pub const CELESTIAL_PASS_PRODUCT_ID: &str = "com.gravipop.starpass";
