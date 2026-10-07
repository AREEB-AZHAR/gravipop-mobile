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

// ── Abilities (earn 1 charge per milestone merge, max 3 active) ───────────────
pub const MAX_ABILITY_CHARGES: u32 = 3;
pub const ABILITY_EARN_TIER: usize = 3; // GasGiant or higher grants a charge

// ── Economy / Ads ─────────────────────────────────────────────────────────────
pub const STARDUST_PER_FUSION_BASE: u32 = 5;
pub const REMOVE_ADS_PRODUCT_ID: &str = "com.gravipop.removeads";
pub const CELESTIAL_PASS_PRODUCT_ID: &str = "com.gravipop.starpass";
/// Infrequent post-session ad cadence
pub const INTERSTITIAL_RUN_INTERVAL: u32 = 5;
pub const FREE_REVIVES_PER_RUN: u32 = 1;
