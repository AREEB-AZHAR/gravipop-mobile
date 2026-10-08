pub mod audio;
pub mod core;
pub mod graphics;
pub mod monetization;
pub mod physics;
pub mod ui;

use crate::audio::AudioEngine;
use crate::core::config::*;
use crate::core::game_state::*;
use crate::core::leaderboard::LeaderboardClient;
use crate::core::save_system::{SaveData, SaveManager};
use crate::core::sector::{all_sectors, ObjectiveTracker};
use crate::graphics::*;
use crate::monetization::ads::{AdOutcome, PlatformAdService, RewardType};
use crate::monetization::ad_bridge;
use crate::monetization::billing::{BillingService, MockBillingService};
use crate::monetization::economy::EconomyCatalog;
use crate::physics::*;
use crate::ui::*;
use macroquad::prelude::*;

// ─────────────────────────────────────────────────────────────────────────────
//  Window configuration
// ─────────────────────────────────────────────────────────────────────────────

pub fn window_conf() -> Conf {
    Conf {
        window_title: "GraviPop: Stellar Conservatory".to_string(),
        window_width: 450,
        window_height: 800,
        window_resizable: true,
        high_dpi: true,
        ..Default::default()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Text & Hit Helpers
// ─────────────────────────────────────────────────────────────────────────────

pub fn draw_centered(text: &str, cx: f32, y: f32, size: f32, color: Color, font: Option<&Font>) {
    let sz = size as u16;
    let dim = measure_text(text, font, sz, 1.0);
    draw_text_ex(
        text,
        cx - dim.width * 0.5,
        y,
        TextParams {
            font,
            font_size: sz,
            color,
            ..Default::default()
        },
    );
}

pub fn draw_txt(text: &str, x: f32, y: f32, size: f32, color: Color, font: Option<&Font>) {
    draw_text_ex(
        text,
        x,
        y,
        TextParams {
            font,
            font_size: size as u16,
            color,
            ..Default::default()
        },
    );
}

pub const BUTTON_LOCK_DELAY: f32 = 0.35;

pub fn hit(pos: Vec2, x: f32, y: f32, w: f32, h: f32) -> bool {
    pos.x >= x && pos.x <= x + w && pos.y >= y && pos.y <= y + h
}

fn validate_public_name(value: &str) -> Result<String, &'static str> {
    let name = value.trim();
    if name.chars().count() < 3 {
        Err("Enter at least 3 characters.")
    } else if name.chars().count() > 20 {
        Err("Name is limited to 20 characters.")
    } else if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, ' ' | '_' | '-' | '.'))
    {
        Err("Use letters, numbers, spaces, dots, dashes or underscores.")
    } else {
        Ok(name.to_string())
    }
}

/// The drop guide is a stream of falling light instead of a static laser.  It
/// communicates both the selected planet's colour and the direction of travel.
fn draw_drop_stream(x: f32, start_y: f32, end_y: f32, color: Color, active: bool) {
    let spacing = 22.0;
    let phase = (get_time() as f32 * 90.0) % spacing;
    let alpha = if active { 0.92 } else { 0.56 };
    let mut y = start_y - phase;
    let mut index = 0usize;
    while y < end_y {
        if y >= start_y {
            let progress = ((y - start_y) / (end_y - start_y).max(1.0)).clamp(0.0, 1.0);
            let droplet = 2.2 + progress * 2.0;
            let a = alpha * (0.45 + progress * 0.55);
            draw_circle(x, y, droplet, Color::new(color.r, color.g, color.b, a));
            if index % 2 == 0 {
                draw_circle(x - 1.2, y - 1.2, droplet * 0.32, Color::new(1.0, 1.0, 1.0, a * 0.72));
            }
        }
        y += spacing;
        index += 1;
    }
}

/// An open, crystal-clear pot: only its rim and glass highlights are drawn so
/// the starfield and every planet remain visible through it.
fn draw_clear_pot(danger_timer: f32) {
    let rim_color = if danger_timer > 0.0 {
        Color::new(1.0, 0.34, 0.38, 0.98)
    } else {
        Color::new(0.56, 0.82, 1.0, 0.92)
    };
    let shadow = Color::new(0.06, 0.16, 0.33, 0.34);
    let shine = Color::new(0.82, 0.96, 1.0, 0.82);

    // Exterior glow makes the vessel readable without putting a tint over it.
    draw_line(JAR_LEFT - 4.0, JAR_TOP_LINE, JAR_LEFT - 4.0, JAR_BOTTOM + 8.0, 10.0, shadow);
    draw_line(JAR_RIGHT + 4.0, JAR_TOP_LINE, JAR_RIGHT + 4.0, JAR_BOTTOM + 8.0, 10.0, shadow);
    draw_line(JAR_LEFT - 4.0, JAR_BOTTOM + 5.0, JAR_RIGHT + 4.0, JAR_BOTTOM + 5.0, 12.0, shadow);

    draw_line(JAR_LEFT, JAR_TOP_LINE, JAR_LEFT, JAR_BOTTOM, 3.5, rim_color);
    draw_line(JAR_RIGHT, JAR_TOP_LINE, JAR_RIGHT, JAR_BOTTOM, 3.5, rim_color);
    draw_line(JAR_LEFT, JAR_BOTTOM, JAR_RIGHT, JAR_BOTTOM, 4.5, rim_color);
    draw_line(JAR_LEFT - 2.0, JAR_TOP_LINE, JAR_RIGHT + 2.0, JAR_TOP_LINE, 2.0, rim_color);

    // Narrow inner highlights give the glass a polished edge but keep it transparent.
    draw_line(JAR_LEFT + 10.0, JAR_TOP_LINE + 26.0, JAR_LEFT + 10.0, JAR_BOTTOM - 32.0, 1.2, shine);
    draw_line(JAR_RIGHT - 10.0, JAR_TOP_LINE + 40.0, JAR_RIGHT - 10.0, JAR_BOTTOM - 52.0, 1.0, Color::new(0.75, 0.92, 1.0, 0.46));
    draw_circle(JAR_LEFT + 20.0, JAR_TOP_LINE + 26.0, 3.0, shine);
    draw_circle(JAR_RIGHT - 22.0, JAR_BOTTOM - 38.0, 2.5, Color::new(0.82, 0.96, 1.0, 0.60));
}

#[cfg(target_arch = "wasm32")]
fn sync_web_name_input(
    show: bool,
    bounds: (f32, f32, f32, f32),
    current: &str,
    initialize: bool,
    score: u64,
) -> Option<String> {
    crate::core::web_bridge::sync_name_input(show, bounds, current, initialize, score)
}

// ─────────────────────────────────────────────────────────────────────────────
//  Strategic Abilities
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ability {
    GravityWave,     // Nudge all bodies toward jar centre & upwards with high bounce
    SolarFlare,      // Vaporise the 2 highest clutter bodies
    SuperSolarFlare, // Vaporise all planets strictly below Earth level (Asteroid, Moon)
}

impl Ability {
    pub fn label(self) -> &'static str {
        match self {
            Self::GravityWave => "GRAVITY WAVE",
            Self::SolarFlare => "SOLAR FLARE",
            Self::SuperSolarFlare => "SUPER SOLAR FLARE",
        }
    }
    pub fn description(self) -> &'static str {
        match self {
            Self::GravityWave => "Pull & bounce bodies to center",
            Self::SolarFlare => "Vaporise top 2 clutter bodies",
            Self::SuperSolarFlare => "Vaporise all bodies below Earth",
        }
    }
}

fn apply_ability(
    ability: Ability,
    bodies: &mut Vec<CelestialBody>,
    particles: &mut ParticleEngine,
) {
    match ability {
        Ability::GravityWave => {
            let cx = (JAR_LEFT + JAR_RIGHT) * 0.5;
            for b in bodies.iter_mut() {
                let dx = cx - b.pos.x;
                b.vel.x += dx * 1.60; // Extra power to pull bodies into center
                b.vel.y = b.vel.y.min(-200.0) - 280.0; // High cosmic upward bounce
                particles.spawn_trail(b.pos, Color::new(0.35, 0.85, 1.0, 0.9));
            }
        }
        Ability::SolarFlare => {
            // Find the 2 bodies highest on screen (lowest pos.y)
            let mut indices: Vec<(usize, f32)> = bodies
                .iter()
                .enumerate()
                .map(|(i, b)| (i, b.pos.y))
                .collect();
            indices.sort_by(|a, b2| a.1.partial_cmp(&b2.1).unwrap_or(std::cmp::Ordering::Equal));
            let to_remove: Vec<u64> = indices.iter().take(2).map(|(i, _)| bodies[*i].id).collect();

            for b in bodies.iter() {
                if to_remove.contains(&b.id) {
                    particles.spawn_fusion_burst(b.pos, Color::new(1.0, 0.5, 0.2, 1.0), 5);
                }
            }
            bodies.retain(|b| !to_remove.contains(&b.id));
        }
        Ability::SuperSolarFlare => {
            // Vaporise all planets strictly below Earth level (Asteroid, Moon)
            let mut removed_count = 0;
            for b in bodies.iter() {
                if b.tier < CelestialTier::Terrestrial {
                    particles.spawn_fusion_burst(b.pos, Color::new(1.0, 0.85, 0.25, 1.0), 8);
                    particles.add_floating_text(
                        "VAPORIZED".to_string(),
                        b.pos,
                        Color::new(1.0, 0.90, 0.35, 1.0),
                        24.0,
                    );
                    removed_count += 1;
                }
            }
            bodies.retain(|b| b.tier >= CelestialTier::Terrestrial);
            if removed_count > 0 {
                particles.add_floating_text(
                    format!("SUPER FLARE CLEARED {} PLANETS!", removed_count),
                    Vec2::new((JAR_LEFT + JAR_RIGHT) * 0.5, 420.0),
                    Color::new(1.0, 0.88, 0.30, 1.0),
                    34.0,
                );
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Main Game Loop
// ─────────────────────────────────────────────────────────────────────────────

pub async fn game_main() {
    // ── Font ─────────────────────────────────────────────────────────────────
    let font_path = if cfg!(target_os = "android") {
        "font.ttf"
    } else {
        "assets/font.ttf"
    };
    let font: Option<Font> = load_ttf_font(font_path).await.ok();
    let f: Option<&Font> = font.as_ref();

    // ── Core Systems ─────────────────────────────────────────────────────────
    let save_mgr = SaveManager::new();
    let mut save_data = save_mgr.load();
    let mut leaderboard = LeaderboardClient::default();
    let mut name_focused = false;
    let mut leaderboard_status = String::new();
    #[cfg(target_arch = "wasm32")]
    let mut name_prompt_open = false;
    let mut physics_world = PhysicsWorld::default();
    let audio = AudioEngine::new().await;
    let mut starfield = Starfield::new();
    let mut particles = ParticleEngine::new();
    let mut ads = PlatformAdService::new();
    let mut ad_status = String::new();
    let mut billing =
        MockBillingService::new(save_data.ads_removed, save_data.celestial_pass_unlocked);
    let catalog = EconomyCatalog::new();
    let sectors = all_sectors();

    // ── High-Resolution Render Target (Configured by ResolutionProfile) ─────────
    let (init_w, init_h) = save_data.resolution_profile.dimensions();
    let mut target_w = init_w;
    let mut target_h = init_h;
    let mut vt = render_target(target_w, target_h);
    vt.texture.set_filter(FilterMode::Linear);
    let mut vcam = Camera2D {
        target: vec2(VIRTUAL_WIDTH * 0.5, VIRTUAL_HEIGHT * 0.5),
        zoom: vec2(2.0 / VIRTUAL_WIDTH, 2.0 / VIRTUAL_HEIGHT),
        offset: vec2(0.0, 0.0),
        rotation: 0.0,
        render_target: Some(vt.clone()),
        viewport: None,
    };

    // ── Global Session State ─────────────────────────────────────────────────
    let mut game_state = GameState::MainMenu;
    let mut current_sector: usize = 0;
    let mut selected_chapter: usize = 0; // 0: Nebula Rim (1-5), 1: Frost Expanse (6-10), 2: Void (11-15)
    let mut bodies: Vec<CelestialBody> = Vec::new();
    let mut next_body_id = 1u64;
    let mut current_score = 0u64;
    let mut run_stardust = 0u64;
    let mut revives_used = 0u32;
    let mut stardust_doubled = false;
    let mut shop_msg: Option<String> = None;

    // Legacy sector data stays available for the map/shop screens, but gameplay is endless.
    let mut objective = ObjectiveTracker::new(sectors[0].objective.clone());

    // Danger timer (settled bodies overflowing above JAR_TOP_LINE)
    let mut danger_timer = 0.0f32;

    // One random planet at a time — no preview choice or reserve system.
    let mut drop_pool = DropPool::default();
    let mut next_tier = drop_pool.random_tier();
    let mut drop_x = VIRTUAL_WIDTH * 0.5;
    let mut is_aiming = false;
    let mut drop_cooldown = 0.0f32;
    let mut button_lock_timer = 0.0f32;
    let mut require_touch_release = false;
    let mut shop_return_state = GameState::MainMenu;

    // Strategic abilities & Powerups
    let mut ability_charges: u32 = 0;
    let mut ability_flash: f32 = 0.0;
    let mut super_flare_charges: u32 = 0;
    let mut super_flare_milestone: u64 = 0;
    let mut gravity_wave_grace_timer: f32 = 0.0f32;


    // Sector completion celebration
    let mut sector_complete_timer = 0.0f32;
    let earned_stars = 0u8;
    let mut last_pointer_pos = Vec2::new(VIRTUAL_WIDTH * 0.5, VIRTUAL_HEIGHT * 0.5);

    // ─────────────────────────────────────────────────────────────────────────
    //  MAIN LOOP
    // ─────────────────────────────────────────────────────────────────────────
    loop {
        let dt = get_frame_time().min(0.05);

        // ── Input & Coordinates Normalization ────────────────────────────────
        let sw = screen_width();
        let sh = screen_height();
        let scale = (sw / VIRTUAL_WIDTH).min(sh / VIRTUAL_HEIGHT);
        let ox = (sw - VIRTUAL_WIDTH * scale) * 0.5;
        let oy = (sh - VIRTUAL_HEIGHT * scale) * 0.5;

        let (mx, my) = mouse_position();
        let vm = Vec2::new((mx - ox) / scale, (my - oy) / scale);

        let touch_list = touches();
        let dpi = macroquad::miniquad::window::dpi_scale().max(1.0);

        let touch_started = touch_list.iter().any(|t| t.phase == TouchPhase::Started);
        let touch_ended = touch_list.iter().any(|t| t.phase == TouchPhase::Ended);
        let touch_down = !touch_list.is_empty();

        let mouse_pressed = is_mouse_button_pressed(MouseButton::Left);
        let mouse_down = is_mouse_button_down(MouseButton::Left);
        let mouse_released = is_mouse_button_released(MouseButton::Left);

        // Normalize touch coordinate by dpi_scale so it aligns with sw/sh viewport space
        let current_ptr = if let Some(t) = touch_list.first() {
            let tx = t.position.x / dpi;
            let ty = t.position.y / dpi;
            Vec2::new((tx - ox) / scale, (ty - oy) / scale)
        } else if mouse_pressed || mouse_down || mouse_released || (mx != 0.0 || my != 0.0) {
            vm
        } else {
            last_pointer_pos
        };

        if mouse_pressed || mouse_down || touch_down || touch_started || touch_ended {
            last_pointer_pos = current_ptr;
        }

        let ptr = current_ptr;

        let tap = mouse_pressed || touch_started;
        let is_held = mouse_down || touch_down;
        let just_released = mouse_released || touch_ended;

        // Button lock cooldown timer (prevents rapid double-clicks and ghost touch triggers)
        if button_lock_timer > 0.0 {
            button_lock_timer = (button_lock_timer - dt).max(0.0);
        }

        // Mobile touch release guard:
        // After any button press or screen transition, the finger must be completely lifted
        // before any new UI button can be tapped or gameplay dropped/aimed.
        if require_touch_release {
            if is_held {
                button_lock_timer = button_lock_timer.max(BUTTON_LOCK_DELAY);
            } else {
                require_touch_release = false;
            }
        }

        let can_click_ui = button_lock_timer <= 0.0 && !require_touch_release;
        let mut ui_tap = tap && can_click_ui;

        // ── Global Ticks ─────────────────────────────────────────────────────
        starfield.update(dt);
        particles.update(dt);
        if ability_flash > 0.0 {
            ability_flash -= dt;
        }
        ad_bridge::set_removed(billing.is_ad_removed());
        if let Some(outcome) = ads.update() {
            match outcome {
                AdOutcome::RewardEarned(reward) => {
                    ad_status.clear();
                    handle_reward(
                        reward,
                        &mut current_score,
                        &mut run_stardust,
                        &mut revives_used,
                        &mut stardust_doubled,
                        &mut bodies,
                        &mut danger_timer,
                        &mut save_data,
                        &save_mgr,
                        &mut game_state,
                    );
                }
                AdOutcome::Closed => {
                    ad_status.clear();
                    game_state = GameState::GameOver;
                }
                AdOutcome::RewardSkipped => {
                    ad_status = "Ad closed early. No reward earned.".to_string();
                    game_state = GameState::GameOver;
                }
                AdOutcome::Unavailable => {
                    ad_status = "Ad unavailable. Please try again later.".to_string();
                    game_state = GameState::GameOver;
                }
            }
        }

        // ── LOGIC ────────────────────────────────────────────────────────────
        let state_before_frame = game_state;
        match game_state {
            // ── Main Menu ────────────────────────────────────────────────────
            GameState::MainMenu => {
                if ad_bridge::privacy_options_required()
                    && ui_tap
                    && hit(ptr, VIRTUAL_WIDTH - 210.0, 20.0, 190.0, 44.0)
                {
                    ui_tap = false;
                    button_lock_timer = BUTTON_LOCK_DELAY;
                    ad_bridge::show_privacy_options();
                }
                let cx = VIRTUAL_WIDTH * 0.5;
                let bw = 360.0;
                let bx = cx - bw * 0.5;
                let play_y = VIRTUAL_HEIGHT * 0.49;
                let ach_y = play_y + 82.0;
                let set_y = ach_y + 72.0;
                let lb_y = set_y + 72.0;

                if (ui_tap && hit(ptr, bx, play_y, bw, 74.0))
                    || is_key_pressed(KeyCode::Space)
                    || is_key_pressed(KeyCode::Enter)
                {
                    ui_tap = false;
                    require_touch_release = true;
                    button_lock_timer = BUTTON_LOCK_DELAY;
                    shop_return_state = GameState::MainMenu;
                    ad_status.clear();
                    bodies.clear();
                    next_body_id = 1;
                    current_score = 0;
                    run_stardust = 0;
                    revives_used = 0;
                    stardust_doubled = false;
                    danger_timer = 0.0;
                    ability_charges = 0;
                    super_flare_charges = 0;
                    super_flare_milestone = 0;
                    gravity_wave_grace_timer = 0.0;
                    drop_cooldown = 0.0;
                    drop_pool = DropPool::default();
                    next_tier = drop_pool.random_tier();
                    is_aiming = false;
                    game_state = GameState::Playing;
                }
                // Achievements button
                if ui_tap && hit(ptr, bx, ach_y, bw, 64.0) {
                    ui_tap = false;
                    require_touch_release = true;
                    button_lock_timer = BUTTON_LOCK_DELAY;
                    shop_return_state = GameState::MainMenu;
                    game_state = GameState::Achievements;
                }
                // Settings button
                if ui_tap && hit(ptr, bx, set_y, bw, 64.0) {
                    ui_tap = false;
                    require_touch_release = true;
                    button_lock_timer = BUTTON_LOCK_DELAY;
                    shop_return_state = GameState::MainMenu;
                    game_state = GameState::Settings;
                }
                // Leaderboard button
                if ui_tap && hit(ptr, bx, lb_y, bw, 62.0) {
                    ui_tap = false;
                    require_touch_release = true;
                    button_lock_timer = BUTTON_LOCK_DELAY;
                    leaderboard.refresh().await;
                    leaderboard_status = leaderboard.status.clone();
                    game_state = GameState::Leaderboard;
                }
            }

            GameState::GameOver => {
                // Name field is deliberately available after every run.
                if ui_tap && hit(ptr, 50.0, 531.0, 620.0, 48.0) {
                    ui_tap = false;
                    name_focused = true;
                    #[cfg(target_os = "android")]
                    macroquad::miniquad::window::show_keyboard(true);
                }
                if name_focused {
                    if is_key_pressed(KeyCode::Backspace) {
                        save_data.public_name.pop();
                    }
                    if let Some(ch) = get_char_pressed() {
                        if ch.is_ascii_alphanumeric() || matches!(ch, ' ' | '_' | '-' | '.') {
                            if save_data.public_name.chars().count() < 20 {
                                save_data.public_name.push(ch);
                            }
                        }
                    }
                }
            }

            GameState::Leaderboard => {
                if (ui_tap && hit(ptr, 20.0, 20.0, 110.0, 54.0)) || is_key_pressed(KeyCode::Escape) {
                    ui_tap = false;
                    require_touch_release = true;
                    button_lock_timer = BUTTON_LOCK_DELAY;
                    game_state = GameState::MainMenu;
                }
                if ui_tap && hit(ptr, VIRTUAL_WIDTH - 170.0, 20.0, 150.0, 54.0) {
                    ui_tap = false;
                    button_lock_timer = BUTTON_LOCK_DELAY;
                    leaderboard.refresh().await;
                    leaderboard_status = leaderboard.status.clone();
                }
            }

            // ── Galaxy Map (3 Chapter Tabs) ──────────────────────────────────
            GameState::GalaxyMap => {
                // Back button (top left)
                if (ui_tap && hit(ptr, 20.0, 20.0, 90.0, 50.0)) || is_key_pressed(KeyCode::Escape) {
                    ui_tap = false;
                    require_touch_release = true;
                    button_lock_timer = BUTTON_LOCK_DELAY;
                    game_state = GameState::MainMenu;
                }

                // Chapter Tab Switcher (y: 125..175)
                let tab_y = 125.0;
                let tab_w = 210.0;
                let tab_h = 48.0;
                let tab_gap = 15.0;
                let tab_start_x = (VIRTUAL_WIDTH - (tab_w * 3.0 + tab_gap * 2.0)) * 0.5;

                for c in 0..3 {
                    let tx = tab_start_x + c as f32 * (tab_w + tab_gap);
                    if ui_tap && hit(ptr, tx, tab_y, tab_w, tab_h) {
                        ui_tap = false;
                        button_lock_timer = BUTTON_LOCK_DELAY;
                        selected_chapter = c;
                    }
                }

                // Keyboard chapter switching (Q / E or 1 / 2 / 3)
                if is_key_pressed(KeyCode::Key1) {
                    selected_chapter = 0;
                }
                if is_key_pressed(KeyCode::Key2) {
                    selected_chapter = 1;
                }
                if is_key_pressed(KeyCode::Key3) {
                    selected_chapter = 2;
                }

                // Sector cards in current chapter (5 cards per chapter)
                let start_idx = selected_chapter * 5;
                for row in 0..5 {
                    let idx = start_idx + row;
                    if idx >= sectors.len() {
                        break;
                    }
                    let card_y = 195.0 + row as f32 * 115.0;
                    let card_x = 40.0;
                    let card_w = VIRTUAL_WIDTH - 80.0;
                    let card_h = 100.0;

                    if save_data.is_sector_unlocked(idx)
                        && ui_tap
                        && hit(ptr, card_x, card_y, card_w, card_h)
                    {
                        ui_tap = false;
                        require_touch_release = true;
                        button_lock_timer = BUTTON_LOCK_DELAY;
                        shop_return_state = GameState::MainMenu;
                        current_sector = idx;
                        let sec = &sectors[current_sector];
                        objective = ObjectiveTracker::new(sec.objective.clone());
                        bodies.clear();
                        next_body_id = 1;
                        current_score = 0;
                        run_stardust = 0;
                        revives_used = 0;
                        stardust_doubled = false;
                        danger_timer = 0.0;
                        ability_charges = 0;
                        drop_cooldown = 0.0;
                        drop_pool = DropPool::default();
                        next_tier = drop_pool.random_tier();
                        is_aiming = false;
                        game_state = GameState::Playing;
                        break;
                    }
                }
            }

            // ── Playing ──────────────────────────────────────────────────────
            GameState::Playing => {
                let active_tier = next_tier;
                let r = active_tier.radius();

                // Drop cooldown timer
                if drop_cooldown > 0.0 {
                    drop_cooldown -= dt;
                }

                // Pause button (top right: x: 640..700, y: 20..68)
                if (ui_tap && hit(ptr, VIRTUAL_WIDTH - 80.0, 20.0, 60.0, 48.0))
                    || is_key_pressed(KeyCode::P)
                    || is_key_pressed(KeyCode::Escape)
                {
                    ui_tap = false;
                    require_touch_release = true;
                    button_lock_timer = BUTTON_LOCK_DELAY;
                    game_state = GameState::Paused;
                }

                // Keyboard Steering & Drop
                if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
                    drop_x = (drop_x - 520.0 * dt).clamp(JAR_LEFT + r, JAR_RIGHT - r);
                    is_aiming = true;
                }
                if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
                    drop_x = (drop_x + 520.0 * dt).clamp(JAR_LEFT + r, JAR_RIGHT - r);
                    is_aiming = true;
                }

                let mut clicked_ui = false;
                let in_pause_btn = hit(ptr, VIRTUAL_WIDTH - 80.0, 20.0, 60.0, 48.0);
                let ab_y = 1136.0;
                let ab_w = 170.0;
                let ab_h = 66.0;
                let in_wave_btn = hit(ptr, JAR_LEFT, ab_y, ab_w, ab_h);
                let flare_x = JAR_LEFT + ab_w + 15.0; // 275.0
                let in_flare_btn = hit(ptr, flare_x, ab_y, ab_w, ab_h);
                let super_x = JAR_LEFT + (ab_w + 15.0) * 2.0; // 460.0
                let in_super_btn = hit(ptr, super_x, ab_y, ab_w, ab_h);

                if in_pause_btn || in_wave_btn || in_flare_btn || in_super_btn {
                    clicked_ui = true;
                }

                if ui_tap {
                    // Powerup 1: Gravity Wave (High bounce, center pull, overflow immunity)
                    if in_wave_btn && ability_charges > 0 {
                        ui_tap = false;
                        apply_ability(Ability::GravityWave, &mut bodies, &mut particles);
                        danger_timer = 0.0;
                        gravity_wave_grace_timer = GRAVITY_WAVE_GRACE_DURATION;
                        audio.play_fusion_chime(4);
                        ability_charges -= 1;
                        button_lock_timer = BUTTON_LOCK_DELAY;
                    } else if in_flare_btn && ability_charges > 0 {
                        // Powerup 2: Solar Flare (Vaporise top 2 clutter bodies)
                        ui_tap = false;
                        apply_ability(Ability::SolarFlare, &mut bodies, &mut particles);
                        audio.play_fusion_chime(6);
                        ability_charges -= 1;
                        button_lock_timer = BUTTON_LOCK_DELAY;
                    } else if in_super_btn && super_flare_charges > 0 {
                        // Powerup 3: SUPER SOLAR FLARE (Vaporise all bodies below Earth)
                        ui_tap = false;
                        apply_ability(Ability::SuperSolarFlare, &mut bodies, &mut particles);
                        audio.play_fusion_chime(8);
                        super_flare_charges -= 1;
                        button_lock_timer = BUTTON_LOCK_DELAY;
                    }
                }

                // Keyboard Hotkeys: 1, 2, 3
                if is_key_pressed(KeyCode::Key1) && ability_charges > 0 {
                    apply_ability(Ability::GravityWave, &mut bodies, &mut particles);
                    danger_timer = 0.0;
                    gravity_wave_grace_timer = GRAVITY_WAVE_GRACE_DURATION;
                    audio.play_fusion_chime(4);
                    ability_charges -= 1;
                } else if is_key_pressed(KeyCode::Key2) && ability_charges > 0 {
                    apply_ability(Ability::SolarFlare, &mut bodies, &mut particles);
                    audio.play_fusion_chime(6);
                    ability_charges -= 1;
                } else if is_key_pressed(KeyCode::Key3) && super_flare_charges > 0 {
                    apply_ability(Ability::SuperSolarFlare, &mut bodies, &mut particles);
                    audio.play_fusion_chime(8);
                    super_flare_charges -= 1;
                }

                // Touch / Mouse Aiming inside Jar Area
                let in_aim_zone = ptr.x >= JAR_LEFT - 20.0
                    && ptr.x <= JAR_RIGHT + 20.0
                    && ptr.y >= 108.0
                    && ptr.y <= JAR_BOTTOM;

                if is_held && in_aim_zone && !clicked_ui && !require_touch_release {
                    drop_x = ptr.x.clamp(JAR_LEFT + r, JAR_RIGHT - r);
                    is_aiming = true;
                }

                // Commit Drop on Release or Spacebar
                let keyboard_drop = is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Down);
                let commit_drop = (just_released && is_aiming && !clicked_ui && !require_touch_release) || keyboard_drop;

                if commit_drop && drop_cooldown <= 0.0 {
                    let tier = next_tier;
                    drop_x = drop_x.clamp(JAR_LEFT + tier.radius(), JAR_RIGHT - tier.radius());
                    let body = CelestialBody::new(
                        next_body_id,
                        tier,
                        Vec2::new(drop_x, DROP_Y),
                        Vec2::new(0.0, 140.0),
                    );
                    next_body_id += 1;
                    bodies.push(body);
                    audio.play_slingshot();
                    particles.spawn_trail(Vec2::new(drop_x, DROP_Y), tier.primary_color());

                    // Roll exactly one new random planet after each drop.
                    next_tier = drop_pool.random_tier();
                    drop_cooldown = DROP_COOLDOWN;
                    is_aiming = false;
                }

                // Physics Simulation Step
                let fusions = physics_world.step(&mut bodies, dt);
                for fus in fusions {
                    drop_pool.record_merge(fus.new_tier);
                    current_score += fus.score_awarded;
                    run_stardust += fus.stardust_awarded as u64;

                    // Record merged tier for achievements
                    let tier_idx = fus.new_tier as usize;
                    if !save_data.merged_tiers.contains(&tier_idx) {
                        save_data.merged_tiers.push(tier_idx);
                    }

                    // Milestone merges grant standard Ability Charges
                    if (fus.new_tier as usize) >= ABILITY_EARN_TIER
                        && ability_charges < MAX_ABILITY_CHARGES
                    {
                        ability_charges += 1;
                        ability_flash = 0.9;
                    }

                    // Super Solar Flare Charge: Earned every 50,000 score achieved (max 2 charges)
                    while current_score >= super_flare_milestone + SUPER_FLARE_SCORE_INTERVAL {
                        super_flare_milestone += SUPER_FLARE_SCORE_INTERVAL;
                        if super_flare_charges < MAX_SUPER_FLARE_CHARGES {
                            super_flare_charges += 1;
                            particles.add_floating_text(
                                "SUPER SOLAR FLARE READY!".to_string(),
                                Vec2::new(VIRTUAL_WIDTH * 0.5, 480.0),
                                Color::new(1.0, 0.85, 0.25, 1.0),
                                32.0,
                            );
                            audio.play_fusion_chime(7);
                        }
                    }

                    // Check achievements for score milestones up to 1,000,000 & all merges
                    let newly_unlocked = crate::core::achievements::check_achievements(
                        current_score,
                        &save_data.merged_tiers,
                        &mut save_data.unlocked_achievements,
                    );
                    for ach in newly_unlocked {
                        particles.add_floating_text(
                            format!("🏆 {} UNLOCKED!", ach.title),
                            Vec2::new(VIRTUAL_WIDTH * 0.5, 410.0),
                            Color::new(1.0, 0.90, 0.35, 1.0),
                            30.0,
                        );
                        audio.play_fusion_chime(8);
                        let _ = save_mgr.save(&save_data);
                    }

                    objective.on_merge(fus.new_tier);
                    objective.score_progress = current_score;
                    particles.spawn_fusion_burst(
                        fus.pos,
                        fus.new_tier.primary_color(),
                        fus.new_tier as usize,
                    );
                    let pts_label = format!("MERGE +{}", fus.score_awarded);
                    particles.add_floating_text(
                        pts_label,
                        fus.pos,
                        fus.new_tier.glow_color(),
                        28.0,
                    );
                    audio.play_fusion_chime(fus.new_tier as usize);

                    if (fus.new_tier as usize) >= 5 {
                        let name = format!("{} FORMED!", fus.new_tier.name().to_uppercase());
                        particles.add_floating_text(
                            name,
                            fus.pos - Vec2::new(0.0, 50.0),
                            fus.new_tier.label_color(),
                            34.0,
                        );
                    }
                }
                objective.score_progress = current_score;

                // Live high score tracking
                if current_score > save_data.high_score {
                    save_data.high_score = current_score;
                }

                // Danger overflow handling
                // When Powerup 1 (Gravity Wave) is used, do not check for overflows during grace period
                if gravity_wave_grace_timer > 0.0 {
                    gravity_wave_grace_timer -= dt;
                    danger_timer = 0.0;
                } else {
                    let any_danger = bodies.iter().any(|b| b.is_overflowing());
                    if any_danger {
                        danger_timer += dt;
                        if danger_timer >= DANGER_TIME {
                            danger_timer = DANGER_TIME;
                            audio.play_game_over();
                            save_data.runs_played += 1;
                            save_data.stardust += run_stardust;
                            let _ = save_mgr.save(&save_data);
                            if !billing.is_ad_removed()
                                && save_data.runs_played.is_multiple_of(INTERSTITIAL_RUN_INTERVAL)
                                && ads.start_interstitial_ad()
                            {
                                game_state = GameState::WatchingAd;
                            } else {
                                game_state = GameState::GameOver;
                            }
                        }
                    } else if danger_timer > 0.0 {
                        danger_timer = (danger_timer - dt * 1.5).max(0.0);
                    }
                }
            }

            // ── Sector Complete ──────────────────────────────────────────────
            GameState::SectorComplete => {
                sector_complete_timer += dt;
                if ui_tap && hit(ptr, 90.0, 674.0, 540.0, 48.0) {
                    ui_tap = false;
                    name_focused = true;
                    #[cfg(target_os = "android")]
                    macroquad::miniquad::window::show_keyboard(true);
                }
                if name_focused {
                    if is_key_pressed(KeyCode::Backspace) {
                        save_data.public_name.pop();
                    }
                    if let Some(ch) = get_char_pressed() {
                        if (ch.is_ascii_alphanumeric() || matches!(ch, ' ' | '_' | '-' | '.'))
                            && save_data.public_name.chars().count() < 20
                        {
                            save_data.public_name.push(ch);
                        }
                    }
                }
                let cx = VIRTUAL_WIDTH * 0.5;
                let nx_y = VIRTUAL_HEIGHT * 0.65;
                let btn_avail = current_sector + 1 < sectors.len();

                // Next Sector button
                if btn_avail
                    && (ui_tap && hit(ptr, cx - 150.0, nx_y, 300.0, 66.0)
                        || is_key_pressed(KeyCode::Space))
                {
                    ui_tap = false;
                    require_touch_release = true;
                    button_lock_timer = BUTTON_LOCK_DELAY;
                    shop_return_state = GameState::MainMenu;
                    current_sector += 1;
                    let sec = &sectors[current_sector];
                    objective = ObjectiveTracker::new(sec.objective.clone());
                    bodies.clear();
                    next_body_id = 1;
                    current_score = 0;
                    run_stardust = 0;
                    revives_used = 0;
                    stardust_doubled = false;
                    danger_timer = 0.0;
                    ability_charges = 0;
                    drop_cooldown = 0.0;
                    drop_pool = DropPool::default();
                    next_tier = drop_pool.random_tier();
                    is_aiming = false;
                    game_state = GameState::Playing;
                }

                // Galaxy Map button
                let mp_y = nx_y + 84.0;
                if ui_tap && hit(ptr, cx - 150.0, mp_y, 300.0, 60.0) {
                    ui_tap = false;
                    require_touch_release = true;
                    button_lock_timer = BUTTON_LOCK_DELAY;
                    game_state = GameState::GalaxyMap;
                }
            }

            // ── Paused ───────────────────────────────────────────────────────
            GameState::Paused => {
                let cx = VIRTUAL_WIDTH * 0.5;
                let card_y = VIRTUAL_HEIGHT * 0.26;
                let bw = 380.0;
                let bx = cx - bw * 0.5;
                let ry = card_y + 242.0;
                let qy = ry + 80.0;

                if (ui_tap && hit(ptr, bx, ry, bw, 66.0))
                    || is_key_pressed(KeyCode::P)
                    || is_key_pressed(KeyCode::Escape)
                    || is_key_pressed(KeyCode::Space)
                {
                    ui_tap = false;
                    require_touch_release = true;
                    button_lock_timer = BUTTON_LOCK_DELAY;
                    game_state = GameState::Playing;
                }
                if ui_tap && hit(ptr, bx, qy, bw, 66.0) {
                    ui_tap = false;
                    require_touch_release = true;
                    button_lock_timer = BUTTON_LOCK_DELAY;
                    if current_score > 0 {
                        danger_timer = DANGER_TIME;
                        save_data.runs_played += 1;
                        save_data.stardust += run_stardust;
                        if current_score > save_data.high_score {
                            save_data.high_score = current_score;
                        }
                        let _ = save_mgr.save(&save_data);
                        name_focused = true;
                        #[cfg(target_os = "android")]
                        macroquad::miniquad::window::show_keyboard(true);
                        shop_return_state = GameState::GameOver;
                        game_state = GameState::GameOver;
                    } else {
                        current_score = 0;
                        run_stardust = 0;
                        shop_return_state = GameState::MainMenu;
                        game_state = GameState::MainMenu;
                    }
                }
            }

            _ => {}
        }

        #[cfg(target_arch = "wasm32")]
        {
            let show = matches!(game_state, GameState::GameOver | GameState::SectorComplete);
            let opening = show && !name_prompt_open;
            name_prompt_open = show;
            let bounds = if game_state == GameState::GameOver {
                (50.0, 531.0, 620.0, 48.0)
            } else {
                (90.0, 674.0, 540.0, 48.0)
            };
            if let Some(value) =
                sync_web_name_input(show, bounds, &save_data.public_name, opening, current_score)
            {
                save_data.public_name = value
                    .chars()
                    .filter(|c| c.is_ascii_alphanumeric() || matches!(c, ' ' | '_' | '-' | '.'))
                    .take(20)
                    .collect();
            }

        }
        leaderboard.poll();
        if !leaderboard.status.is_empty() {
            leaderboard_status = leaderboard.status.clone();
        }
        #[cfg(target_os = "android")]
        if !matches!(game_state, GameState::GameOver | GameState::SectorComplete) && name_focused {
            macroquad::miniquad::window::show_keyboard(false);
            name_focused = false;
        }

        // Recreate render target when profile dimensions change
        let (res_w, res_h) = save_data.resolution_profile.dimensions();
        let current_target_w = res_w;
        let current_target_h = res_h;
        if target_w != current_target_w || target_h != current_target_h {
            target_w = current_target_w;
            target_h = current_target_h;
            vt = render_target(target_w, target_h);
            vt.texture.set_filter(FilterMode::Linear);
            vcam = Camera2D {
                target: vec2(VIRTUAL_WIDTH * 0.5, VIRTUAL_HEIGHT * 0.5),
                zoom: vec2(2.0 / VIRTUAL_WIDTH, 2.0 / VIRTUAL_HEIGHT),
                offset: vec2(0.0, 0.0),
                rotation: 0.0,
                render_target: Some(vt.clone()),
                viewport: None,
            };
        }

        // ── RENDER (1:1 High-Resolution Projection) ───────────────────────────
        set_camera(&vcam);
        clear_background(Color::new(0.02, 0.02, 0.06, 1.0));

        let danger_ratio = (danger_timer / CRITICAL_TIME_LIMIT).clamp(0.0, 1.0);
        starfield.draw(danger_ratio);

        let mut go_action = GameOverAction::None;
        let mut shop_action = ShopAction::None;
        let mut sector_submit = false;
        let web_submit = crate::core::web_bridge::take_submit_request();

        match game_state {
            // ── Main Menu ────────────────────────────────────────────────────
            GameState::MainMenu => {
                let cx = VIRTUAL_WIDTH * 0.5;

                // Ad Privacy pill (if needed by consent)
                if ad_bridge::privacy_options_required() {
                    let ap_hov = hit(ptr, VIRTUAL_WIDTH - 210.0, 20.0, 190.0, 44.0);
                    draw_rectangle(
                        VIRTUAL_WIDTH - 210.0,
                        20.0,
                        190.0,
                        44.0,
                        if ap_hov { Color::new(0.18, 0.26, 0.42, 0.95) } else { Color::new(0.10, 0.14, 0.24, 0.85) },
                    );
                    draw_rectangle_lines(VIRTUAL_WIDTH - 210.0, 20.0, 190.0, 44.0, 1.4, Color::new(0.40, 0.70, 1.0, 0.60));
                    draw_centered("AD PRIVACY", VIRTUAL_WIDTH - 115.0, 48.0, 17.0, WHITE, f);
                }

                // ── Hero Title Card ──
                let card_x = 36.0;
                let card_y = VIRTUAL_HEIGHT * 0.14;
                let card_w = VIRTUAL_WIDTH - 72.0;
                let card_h = 246.0;

                // Outer soft halo
                draw_rectangle(card_x - 3.0, card_y - 3.0, card_w + 6.0, card_h + 6.0, Color::new(0.14, 0.22, 0.52, 0.25));
                // Frosted card fill
                draw_rectangle(card_x, card_y, card_w, card_h, Color::new(0.06, 0.08, 0.20, 0.92));
                // Glowing cosmic borders
                draw_rectangle_lines(card_x, card_y, card_w, card_h, 1.8, Color::new(0.38, 0.72, 1.0, 0.75));
                draw_rectangle_lines(card_x + 4.0, card_y + 4.0, card_w - 8.0, card_h - 8.0, 1.0, Color::new(0.20, 0.35, 0.70, 0.35));

                // Title glow drop-shadow + crisp white text
                draw_centered("GRAVIPOP", cx + 2.0, card_y + 78.0 + 2.0, 80.0, Color::new(0.15, 0.45, 0.95, 0.55), f);
                draw_centered("GRAVIPOP", cx, card_y + 78.0, 80.0, WHITE, f);

                // Subtitle in radiant cyan
                draw_centered("STELLAR CONSERVATORY", cx, card_y + 124.0, 24.0, Color::new(0.42, 0.90, 1.0, 0.98), f);

                // Cosmic separator line with center star
                let div_y = card_y + 146.0;
                draw_line(cx - 150.0, div_y, cx + 150.0, div_y, 1.2, Color::new(0.30, 0.55, 0.85, 0.45));
                draw_circle(cx, div_y, 3.5, Color::new(0.55, 0.92, 1.0, 0.95));

                // All-Time Best Badge
                let best_badge_w = 320.0;
                let best_badge_h = 42.0;
                let best_badge_x = cx - best_badge_w * 0.5;
                let best_badge_y = card_y + 168.0;
                draw_rectangle(best_badge_x, best_badge_y, best_badge_w, best_badge_h, Color::new(0.10, 0.13, 0.30, 0.85));
                draw_rectangle_lines(best_badge_x, best_badge_y, best_badge_w, best_badge_h, 1.4, Color::new(1.0, 0.80, 0.28, 0.75));
                let hi_txt = format!("🏆 ALL-TIME BEST:  {}", save_data.high_score);
                draw_centered(&hi_txt, cx, best_badge_y + 28.0, 20.0, Color::new(1.0, 0.88, 0.35, 1.0), f);

                // ── Interactive Action Buttons ──
                let bw = 360.0;
                let bx = cx - bw * 0.5;
                let play_y = VIRTUAL_HEIGHT * 0.49;
                let play_hov = hit(ptr, bx, play_y, bw, 74.0);

                // 1. PLAY ENDLESS (Celestial Emerald)
                if play_hov {
                    draw_rectangle(bx - 3.0, play_y - 3.0, bw + 6.0, 80.0, Color::new(0.20, 0.82, 0.52, 0.35));
                }
                draw_rectangle(
                    bx,
                    play_y,
                    bw,
                    74.0,
                    if play_hov {
                        Color::new(0.18, 0.75, 0.46, 0.98)
                    } else {
                        Color::new(0.11, 0.58, 0.35, 0.92)
                    },
                );
                draw_rectangle_lines(bx, play_y, bw, 74.0, 2.0, Color::new(0.45, 1.0, 0.70, 0.95));
                draw_line(bx + 16.0, play_y + 3.0, bx + bw - 16.0, play_y + 3.0, 1.5, Color::new(0.65, 1.0, 0.82, 0.70));
                draw_vector_play(bx + 42.0, play_y + 37.0, 26.0, WHITE);
                draw_centered("PLAY ENDLESS", cx + 14.0, play_y + 48.0, 28.0, WHITE, f);

                // 2. ACHIEVEMENTS (Solar Gold)
                let ach_y = play_y + 82.0;
                let ach_hov = hit(ptr, bx, ach_y, bw, 64.0);
                if ach_hov {
                    draw_rectangle(bx - 3.0, ach_y - 3.0, bw + 6.0, 70.0, Color::new(0.95, 0.75, 0.20, 0.30));
                }
                draw_rectangle(
                    bx,
                    ach_y,
                    bw,
                    64.0,
                    if ach_hov {
                        Color::new(0.85, 0.65, 0.18, 0.98)
                    } else {
                        Color::new(0.65, 0.45, 0.10, 0.92)
                    },
                );
                draw_rectangle_lines(bx, ach_y, bw, 64.0, 1.8, Color::new(1.0, 0.88, 0.40, 0.90));
                draw_line(bx + 16.0, ach_y + 3.0, bx + bw - 16.0, ach_y + 3.0, 1.2, Color::new(1.0, 0.92, 0.60, 0.60));
                draw_centered("⭐  COSMIC ACHIEVEMENTS", cx, ach_y + 42.0, 22.0, WHITE, f);

                // 3. SETTINGS & DISPLAY (Quantum Cyan)
                let set_y = ach_y + 72.0;
                let set_hov = hit(ptr, bx, set_y, bw, 64.0);
                if set_hov {
                    draw_rectangle(bx - 3.0, set_y - 3.0, bw + 6.0, 70.0, Color::new(0.20, 0.75, 0.95, 0.30));
                }
                draw_rectangle(
                    bx,
                    set_y,
                    bw,
                    64.0,
                    if set_hov {
                        Color::new(0.18, 0.65, 0.85, 0.98)
                    } else {
                        Color::new(0.10, 0.45, 0.65, 0.92)
                    },
                );
                draw_rectangle_lines(bx, set_y, bw, 64.0, 1.8, Color::new(0.40, 0.85, 1.0, 0.90));
                draw_line(bx + 16.0, set_y + 3.0, bx + bw - 16.0, set_y + 3.0, 1.2, Color::new(0.70, 0.92, 1.0, 0.60));
                draw_centered("⚙️  SETTINGS & DISPLAY", cx, set_y + 42.0, 22.0, WHITE, f);

                // 4. GLOBAL LEADERBOARD (Sapphire Deep)
                let lb_y = set_y + 72.0;
                let lb_hov = hit(ptr, bx, lb_y, bw, 62.0);
                if lb_hov {
                    draw_rectangle(bx - 3.0, lb_y - 3.0, bw + 6.0, 68.0, Color::new(0.20, 0.52, 0.85, 0.30));
                }
                draw_rectangle(
                    bx,
                    lb_y,
                    bw,
                    62.0,
                    if lb_hov {
                        Color::new(0.18, 0.50, 0.80, 0.98)
                    } else {
                        Color::new(0.11, 0.33, 0.58, 0.92)
                    },
                );
                draw_rectangle_lines(bx, lb_y, bw, 62.0, 1.8, Color::new(0.40, 0.80, 1.0, 0.85));
                draw_line(bx + 16.0, lb_y + 3.0, bx + bw - 16.0, lb_y + 3.0, 1.2, Color::new(0.60, 0.88, 1.0, 0.55));
                draw_centered("🏆  GLOBAL LEADERBOARD", cx, lb_y + 40.0, 22.0, WHITE, f);

                // ── Stardust Wallet Badge at Bottom ──
                let dust_w = 300.0;
                let dust_h = 44.0;
                let dust_x = cx - dust_w * 0.5;
                let dust_y = VIRTUAL_HEIGHT - 68.0;
                draw_rectangle(dust_x, dust_y, dust_w, dust_h, Color::new(0.08, 0.10, 0.24, 0.85));
                draw_rectangle_lines(dust_x, dust_y, dust_w, dust_h, 1.2, Color::new(0.35, 0.80, 1.0, 0.50));
                draw_vector_gem(dust_x + 28.0, dust_y + 22.0, 22.0, Color::new(0.35, 0.85, 1.0, 1.0));
                draw_centered(&format!("STARDUST:  {} ✨", save_data.stardust), cx + 10.0, dust_y + 29.0, 20.0, Color::new(0.85, 0.95, 1.0, 1.0), f);

                // Desktop Keyboard Hint
                draw_centered(
                    "Press [SPACE] or Click to Play",
                    cx,
                    VIRTUAL_HEIGHT - 25.0,
                    16.0,
                    Color::new(0.55, 0.55, 0.70, 0.65),
                    f,
                );
            }

            // ── Galaxy Map (Chapter Tabs) ────────────────────────────────────
            GameState::Leaderboard => {
                draw_rectangle(
                    0.0,
                    0.0,
                    VIRTUAL_WIDTH,
                    VIRTUAL_HEIGHT,
                    Color::new(0.01, 0.02, 0.07, 0.98),
                );
                draw_centered(
                    "GLOBAL LEADERBOARD",
                    VIRTUAL_WIDTH * 0.5,
                    94.0,
                    38.0,
                    WHITE,
                    f,
                );
                let back_hov = hit(ptr, 20.0, 20.0, 110.0, 54.0);
                draw_rectangle(
                    20.0,
                    20.0,
                    110.0,
                    54.0,
                    if back_hov {
                        Color::new(0.27, 0.28, 0.46, 1.0)
                    } else {
                        Color::new(0.14, 0.15, 0.28, 1.0)
                    },
                );
                draw_centered("BACK", 75.0, 54.0, 20.0, WHITE, f);
                let refresh_hov = hit(ptr, VIRTUAL_WIDTH - 170.0, 20.0, 150.0, 54.0);
                draw_rectangle(
                    VIRTUAL_WIDTH - 170.0,
                    20.0,
                    150.0,
                    54.0,
                    if refresh_hov {
                        Color::new(0.18, 0.55, 0.78, 1.0)
                    } else {
                        Color::new(0.12, 0.38, 0.62, 1.0)
                    },
                );
                draw_centered("REFRESH", VIRTUAL_WIDTH - 95.0, 54.0, 18.0, WHITE, f);
                for (i, row) in leaderboard.entries.iter().take(20).enumerate() {
                    let y = 145.0 + i as f32 * 48.0;
                    draw_rectangle(
                        52.0,
                        y,
                        VIRTUAL_WIDTH - 104.0,
                        40.0,
                        Color::new(0.08, 0.08, 0.18, 0.92),
                    );
                    draw_txt(
                        &format!("{:02}", i + 1),
                        72.0,
                        y + 27.0,
                        19.0,
                        Color::new(0.55, 0.80, 1.0, 1.0),
                        f,
                    );
                    draw_txt(&row.display_name, 122.0, y + 27.0, 20.0, WHITE, f);
                    let score = format!("{}", row.high_score);
                    let width = measure_text(&score, f, 20, 1.0).width;
                    draw_txt(
                        &score,
                        VIRTUAL_WIDTH - 72.0 - width,
                        y + 27.0,
                        20.0,
                        Color::new(1.0, 0.84, 0.30, 1.0),
                        f,
                    );
                }
                let footer = if !leaderboard_status.is_empty() {
                    leaderboard_status.as_str()
                } else if leaderboard.entries.is_empty() {
                    "No scores yet. Play a run and submit a name."
                } else {
                    "Your best score is saved to your browser identity."
                };
                draw_centered(
                    footer,
                    VIRTUAL_WIDTH * 0.5,
                    VIRTUAL_HEIGHT - 55.0,
                    16.0,
                    Color::new(0.70, 0.75, 0.90, 0.90),
                    f,
                );
            }

            // ── Galaxy Map (Chapter Tabs) ────────────────────────────────────
            GameState::GalaxyMap => {
                let cx = VIRTUAL_WIDTH * 0.5;

                // Header
                draw_centered("GALAXY MAP", cx, 65.0, 36.0, WHITE, f);
                draw_centered(
                    "STELLAR CONSERVATORY",
                    cx,
                    95.0,
                    18.0,
                    Color::new(0.40, 0.85, 1.0, 0.85),
                    f,
                );

                // Back Button (Vector Arrow)
                let b_hov = hit(ptr, 20.0, 20.0, 90.0, 50.0);
                draw_rectangle(
                    20.0,
                    20.0,
                    90.0,
                    50.0,
                    if b_hov {
                        Color::new(0.25, 0.25, 0.40, 0.90)
                    } else {
                        Color::new(0.12, 0.12, 0.24, 0.80)
                    },
                );
                draw_rectangle_lines(20.0, 20.0, 90.0, 50.0, 1.5, Color::new(0.5, 0.5, 0.75, 0.6));
                draw_centered("BACK", 65.0, 52.0, 20.0, WHITE, f);

                // 3 Chapter Tabs (y: 125..175)
                let tab_y = 125.0;
                let tab_w = 210.0;
                let tab_h = 48.0;
                let tab_gap = 15.0;
                let tab_start_x = (VIRTUAL_WIDTH - (tab_w * 3.0 + tab_gap * 2.0)) * 0.5;
                let chapter_names = ["I. NEBULA", "II. FROST", "III. VOID"];

                for (c, &chapter_name) in chapter_names.iter().enumerate() {
                    let tx = tab_start_x + c as f32 * (tab_w + tab_gap);
                    let is_active = selected_chapter == c;
                    let thov = hit(ptr, tx, tab_y, tab_w, tab_h);

                    let bg = if is_active {
                        Color::new(0.25, 0.45, 0.85, 0.95)
                    } else if thov {
                        Color::new(0.18, 0.20, 0.35, 0.85)
                    } else {
                        Color::new(0.10, 0.10, 0.20, 0.75)
                    };
                    draw_rectangle(tx, tab_y, tab_w, tab_h, bg);
                    draw_rectangle_lines(
                        tx,
                        tab_y,
                        tab_w,
                        tab_h,
                        if is_active { 2.0 } else { 1.0 },
                        if is_active {
                            WHITE
                        } else {
                            Color::new(0.4, 0.4, 0.6, 0.5)
                        },
                    );
                    draw_centered(
                        chapter_name,
                        tx + tab_w * 0.5,
                        tab_y + 31.0,
                        18.0,
                        if is_active {
                            WHITE
                        } else {
                            Color::new(0.75, 0.75, 0.88, 0.8)
                        },
                        f,
                    );
                }

                // 5 Sector Cards in Selected Chapter
                let start_idx = selected_chapter * 5;
                for row in 0..5 {
                    let idx = start_idx + row;
                    if idx >= sectors.len() {
                        break;
                    }
                    let sec = &sectors[idx];
                    let card_y = 195.0 + row as f32 * 115.0;
                    let card_x = 40.0;
                    let card_w = VIRTUAL_WIDTH - 80.0;
                    let card_h = 100.0;

                    let unlocked = save_data.is_sector_unlocked(idx);
                    let stars = save_data.stars_for(idx);
                    let chov = unlocked && hit(ptr, card_x, card_y, card_w, card_h);

                    let bg_col = if !unlocked {
                        Color::new(0.07, 0.07, 0.12, 0.60)
                    } else if chov {
                        Color::new(0.16, 0.15, 0.32, 0.95)
                    } else {
                        Color::new(0.10, 0.09, 0.22, 0.88)
                    };

                    draw_rectangle(card_x, card_y, card_w, card_h, bg_col);
                    draw_rectangle_lines(
                        card_x,
                        card_y,
                        card_w,
                        card_h,
                        if chov { 2.0 } else { 1.5 },
                        if chov {
                            Color::new(0.55, 0.80, 1.0, 0.9)
                        } else if unlocked {
                            Color::new(0.35, 0.30, 0.65, 0.6)
                        } else {
                            Color::new(0.20, 0.20, 0.30, 0.4)
                        },
                    );

                    if unlocked {
                        // Title & Lore
                        let title = format!("{}. {}", idx + 1, sec.name);
                        draw_txt(&title, card_x + 20.0, card_y + 34.0, 22.0, WHITE, f);
                        draw_txt(
                            sec.lore,
                            card_x + 20.0,
                            card_y + 58.0,
                            15.0,
                            Color::new(0.65, 0.70, 0.85, 0.75),
                            f,
                        );

                        // Goal label
                        let goal_txt = sec.objective.display_text();
                        draw_txt(
                            &format!("Goal: {}", goal_txt),
                            card_x + 20.0,
                            card_y + 84.0,
                            16.0,
                            Color::new(0.40, 0.90, 0.60, 0.95),
                            f,
                        );

                        // 3 Vector Stars on right
                        let star_start_x = card_x + card_w - 110.0;
                        for s in 0..3 {
                            let star_col = if (s as u8) < stars {
                                Color::new(1.0, 0.88, 0.22, 1.0)
                            } else {
                                Color::new(0.28, 0.28, 0.38, 0.65)
                            };
                            draw_vector_star(
                                star_start_x + s as f32 * 36.0,
                                card_y + 40.0,
                                14.0,
                                star_col,
                            );
                        }
                    } else {
                        // Locked Sector Representation
                        draw_vector_lock(
                            card_x + 60.0,
                            card_y + 50.0,
                            32.0,
                            Color::new(0.50, 0.50, 0.65, 0.60),
                        );
                        let title = format!("{}. {}", idx + 1, sec.name);
                        draw_txt(
                            &title,
                            card_x + 105.0,
                            card_y + 45.0,
                            22.0,
                            Color::new(0.55, 0.55, 0.65, 0.65),
                            f,
                        );
                        draw_txt(
                            "Complete previous sector to unlock",
                            card_x + 105.0,
                            card_y + 72.0,
                            16.0,
                            Color::new(0.45, 0.45, 0.55, 0.55),
                            f,
                        );
                    }
                }

                // Bottom Stars & Stardust Summary
                let mut total_stars = 0u32;
                for i in 0..sectors.len() {
                    total_stars += save_data.stars_for(i) as u32;
                }
                let stars_info =
                    format!("STARS COLLECTED: {} / {}", total_stars, sectors.len() * 3);
                draw_centered(
                    &stars_info,
                    cx,
                    VIRTUAL_HEIGHT - 60.0,
                    20.0,
                    Color::new(1.0, 0.85, 0.30, 0.95),
                    f,
                );
            }

            // ── Playing ──────────────────────────────────────────────────────
            GameState::Playing => {
                // The pot is intentionally open and clear; planets and stars show through it.
                for body in &bodies {
                    BodyRenderer::draw_body(body);
                }
                particles.draw();
                draw_clear_pot(danger_timer);

                // A single friendly next-orb replaces the old two-choice/reserve bar.
                let card_x = VIRTUAL_WIDTH * 0.5 - 126.0;
                let card_y = 102.0;
                let card_hover = hit(ptr, card_x, card_y, 252.0, 92.0);
                let card_color = if card_hover {
                    Color::new(0.10, 0.17, 0.34, 0.92)
                } else {
                    Color::new(0.06, 0.10, 0.24, 0.88)
                };
                draw_rectangle(card_x, card_y, 252.0, 92.0, card_color);
                draw_rectangle_lines(card_x, card_y, 252.0, 92.0, 1.5, Color::new(0.42, 0.68, 1.0, 0.72));
                draw_centered("NEXT DROP", VIRTUAL_WIDTH * 0.5, card_y + 22.0, 14.0, Color::new(0.65, 0.82, 1.0, 0.95), f);
                BodyRenderer::draw_preview(next_tier, Vec2::new(card_x + 48.0, card_y + 57.0), 25.0);
                draw_txt(next_tier.name(), card_x + 88.0, card_y + 62.0, 23.0, WHITE, f);
                draw_txt("drag + release", card_x + 88.0, card_y + 82.0, 14.0, Color::new(0.72, 0.80, 0.96, 0.82), f);

                let r_drop = next_tier.radius();
                draw_drop_stream(
                    drop_x,
                    DROP_Y + r_drop,
                    JAR_BOTTOM - r_drop,
                    next_tier.primary_color(),
                    is_aiming,
                );
                let preview = CelestialBody::new(0, next_tier, Vec2::new(drop_x, DROP_Y), Vec2::ZERO);
                BodyRenderer::draw_body(&preview);

                Hud::draw(current_score, save_data.high_score, save_data.stardust, danger_timer, f);

                // ── Strategic Powerup Controls (3 Buttons: Wave, Flare, Super Solar Flare) ──
                let ab_y = 1136.0;
                let ab_w = 170.0;
                let ab_h = 66.0;

                let powerup_configs = [
                    (
                        JAR_LEFT,
                        "WAVE",
                        "[1]",
                        "Pull & Pop",
                        Color::new(0.25, 0.75, 1.0, 1.0),
                        ability_charges > 0,
                    ),
                    (
                        JAR_LEFT + ab_w + 15.0,
                        "FLARE",
                        "[2]",
                        "Burn Top 2",
                        Color::new(1.0, 0.55, 0.22, 1.0),
                        ability_charges > 0,
                    ),
                    (
                        JAR_LEFT + (ab_w + 15.0) * 2.0,
                        "SUPER FLARE",
                        "[3]",
                        "Wipe < Earth",
                        Color::new(1.0, 0.82, 0.24, 1.0),
                        super_flare_charges > 0,
                    ),
                ];

                for (x, title, key, desc, accent, available) in powerup_configs {
                    let hovered = available && hit(ptr, x, ab_y, ab_w, ab_h);
                    let pulse = if available {
                        ((get_time() as f32 * 6.0).sin() * 0.15 + 0.85).max(0.0)
                    } else {
                        0.35
                    };

                    // Frosted dark glass container
                    let bg = if hovered {
                        Color::new(accent.r * 0.28, accent.g * 0.28, accent.b * 0.28, 0.96)
                    } else if available {
                        Color::new(0.06, 0.08, 0.16, 0.92)
                    } else {
                        Color::new(0.04, 0.05, 0.10, 0.70)
                    };
                    draw_rectangle(x, ab_y, ab_w, ab_h, bg);

                    // Specular top highlight sheen
                    draw_line(
                        x + 4.0,
                        ab_y + 2.0,
                        x + ab_w - 4.0,
                        ab_y + 2.0,
                        1.2,
                        Color::new(1.0, 1.0, 1.0, if available { 0.50 } else { 0.15 }),
                    );

                    // Glowing neon outer ring
                    let border_col = if hovered {
                        accent
                    } else if available {
                        Color::new(accent.r, accent.g, accent.b, 0.80 * pulse)
                    } else {
                        Color::new(0.25, 0.30, 0.40, 0.40)
                    };
                    draw_rectangle_lines(x, ab_y, ab_w, ab_h, if available { 1.8 } else { 1.0 }, border_col);

                    // Title
                    let title_sz = if title == "SUPER FLARE" { 14.0 } else { 16.0 };
                    draw_centered(
                        title,
                        x + ab_w * 0.5,
                        ab_y + 24.0,
                        title_sz,
                        if available { WHITE } else { Color::new(0.60, 0.65, 0.75, 0.65) },
                        f,
                    );

                    // Key pill badge
                    draw_centered(
                        key,
                        x + ab_w * 0.5,
                        ab_y + 42.0,
                        13.0,
                        if available { accent } else { Color::new(0.50, 0.55, 0.65, 0.50) },
                        f,
                    );

                    // Subtext action description
                    draw_centered(
                        desc,
                        x + ab_w * 0.5,
                        ab_y + 57.0,
                        11.0,
                        if available { Color::new(0.85, 0.92, 1.0, 0.85) } else { Color::new(0.40, 0.45, 0.55, 0.50) },
                        f,
                    );
                }

                // ── Dual Usage Meters ─────────────────────────────────────────
                let meter_y = ab_y + ab_h + 16.0;

                // Meter 1: Abilities 1 & 2 Charges (Cosmic Charges)
                let m1_x = JAR_LEFT + 15.0;
                draw_txt(
                    &format!("CHARGES  {}/{}", ability_charges, MAX_ABILITY_CHARGES),
                    m1_x,
                    meter_y,
                    15.0,
                    Color::new(0.40, 0.85, 1.0, 0.90),
                    f,
                );
                // 3 Pips for standard ability charges
                for i in 0..MAX_ABILITY_CHARGES {
                    let pip_x = m1_x + 130.0 + i as f32 * 18.0;
                    let filled = i < ability_charges;
                    draw_circle(
                        pip_x,
                        meter_y - 5.0,
                        5.5,
                        if filled { Color::new(0.35, 0.85, 1.0, 1.0) } else { Color::new(0.18, 0.25, 0.38, 0.60) },
                    );
                    draw_circle_lines(pip_x, meter_y - 5.0, 5.5, 1.0, Color::new(0.40, 0.85, 1.0, 0.70));
                }

                // Meter 2: SUPER SOLAR FLARE (Separate Usage Meter & 50k Score Gauge)
                let m2_x = JAR_RIGHT - 230.0;
                draw_txt(
                    &format!("SUPER FLARE  {}/{}", super_flare_charges, MAX_SUPER_FLARE_CHARGES),
                    m2_x,
                    meter_y,
                    15.0,
                    Color::new(1.0, 0.82, 0.25, 0.95),
                    f,
                );
                // 2 Pips for Super Solar Flare storage
                for i in 0..MAX_SUPER_FLARE_CHARGES {
                    let pip_x = m2_x + 160.0 + i as f32 * 20.0;
                    let filled = i < super_flare_charges;
                    draw_circle(
                        pip_x,
                        meter_y - 5.0,
                        6.5,
                        if filled { Color::new(1.0, 0.82, 0.20, 1.0) } else { Color::new(0.28, 0.22, 0.12, 0.70) },
                    );
                    draw_circle_lines(pip_x, meter_y - 5.0, 6.5, 1.2, Color::new(1.0, 0.85, 0.30, 0.85));
                }

                // 50,000 Score Progress Bar
                let bar_w = 540.0;
                let bar_h = 6.0;
                let bar_x = JAR_LEFT;
                let bar_y = meter_y + 12.0;
                let score_in_bracket = (current_score % SUPER_FLARE_SCORE_INTERVAL) as f32;
                let sf_ratio = if super_flare_charges >= MAX_SUPER_FLARE_CHARGES {
                    1.0
                } else {
                    (score_in_bracket / SUPER_FLARE_SCORE_INTERVAL as f32).clamp(0.0, 1.0)
                };

                draw_rectangle(bar_x, bar_y, bar_w, bar_h, Color::new(0.10, 0.12, 0.20, 0.85));
                draw_rectangle(
                    bar_x,
                    bar_y,
                    bar_w * sf_ratio,
                    bar_h,
                    if super_flare_charges >= MAX_SUPER_FLARE_CHARGES {
                        Color::new(1.0, 0.85, 0.25, 1.0)
                    } else {
                        Color::new(1.0, 0.65, 0.18, 0.90)
                    },
                );
                draw_rectangle_lines(bar_x, bar_y, bar_w, bar_h, 1.0, Color::new(0.40, 0.50, 0.70, 0.40));

                let sf_label = if super_flare_charges >= MAX_SUPER_FLARE_CHARGES {
                    "SUPER FLARE MAX STORAGE REACHED (2/2)".to_string()
                } else {
                    format!("SUPER FLARE CHARGE: {:.1}k / 50k", score_in_bracket / 1000.0)
                };
                draw_centered(&sf_label, VIRTUAL_WIDTH * 0.5, bar_y + 18.0, 12.0, Color::new(0.75, 0.85, 0.98, 0.80), f);
            }

            // ── Sector Complete ──────────────────────────────────────────────
            GameState::SectorComplete => {
                for body in &bodies {
                    BodyRenderer::draw_body(body);
                }
                particles.draw();

                draw_rectangle(
                    0.0,
                    0.0,
                    VIRTUAL_WIDTH,
                    VIRTUAL_HEIGHT,
                    Color::new(0.0, 0.0, 0.0, 0.82),
                );
                let cx = VIRTUAL_WIDTH * 0.5;
                let sec = &sectors[current_sector];

                let pulse = ((sector_complete_timer * 4.0).sin() * 0.08 + 0.96).max(0.0);
                draw_centered(
                    "SECTOR RESTORED!",
                    cx,
                    VIRTUAL_HEIGHT * 0.26,
                    44.0 * pulse,
                    Color::new(0.35, 1.0, 0.55, 1.0),
                    f,
                );
                draw_centered(sec.name, cx, VIRTUAL_HEIGHT * 0.26 + 60.0, 32.0, WHITE, f);
                draw_centered(
                    sec.region,
                    cx,
                    VIRTUAL_HEIGHT * 0.26 + 92.0,
                    20.0,
                    Color::new(0.40, 0.85, 1.0, 0.85),
                    f,
                );

                // 3 Celebratory Vector Stars
                let star_start_x = cx - 72.0;
                for s in 0..3 {
                    let col = if (s as u8) < earned_stars {
                        Color::new(1.0, 0.88, 0.22, 1.0)
                    } else {
                        Color::new(0.28, 0.28, 0.38, 0.60)
                    };
                    draw_vector_star(
                        star_start_x + s as f32 * 72.0,
                        VIRTUAL_HEIGHT * 0.26 + 155.0,
                        26.0,
                        col,
                    );
                }

                let score_str = format!("Final Score: {}", current_score);
                draw_centered(
                    &score_str,
                    cx,
                    VIRTUAL_HEIGHT * 0.44,
                    28.0,
                    Color::new(1.0, 0.85, 0.30, 1.0),
                    f,
                );
                let dust_str = format!("+{} Stardust Earned", run_stardust);
                draw_centered(
                    &dust_str,
                    cx,
                    VIRTUAL_HEIGHT * 0.44 + 40.0,
                    24.0,
                    Color::new(0.40, 0.90, 1.0, 1.0),
                    f,
                );

                draw_centered(
                    "PUBLIC LEADERBOARD NAME",
                    cx,
                    659.0,
                    18.0,
                    Color::new(0.60, 0.88, 1.0, 1.0),
                    f,
                );
                draw_rectangle(90.0, 674.0, 540.0, 48.0, Color::new(0.08, 0.08, 0.16, 1.0));
                draw_rectangle_lines(
                    90.0,
                    674.0,
                    540.0,
                    48.0,
                    1.3,
                    Color::new(0.35, 0.70, 0.95, 0.85),
                );
                let shown_name = if save_data.public_name.is_empty() {
                    "Type a name (3-20 characters)"
                } else {
                    &save_data.public_name
                };
                draw_centered(
                    shown_name,
                    cx,
                    705.0,
                    19.0,
                    if save_data.public_name.is_empty() {
                        Color::new(0.58, 0.60, 0.68, 0.85)
                    } else {
                        WHITE
                    },
                    f,
                );
                if !leaderboard_status.is_empty() {
                    draw_centered(
                        &leaderboard_status,
                        cx,
                        741.0,
                        15.0,
                        Color::new(1.0, 0.68, 0.42, 1.0),
                        f,
                    );
                }
                let submit_hover = hit(ptr, 180.0, 752.0, 360.0, 54.0);
                draw_rectangle(
                    180.0,
                    752.0,
                    360.0,
                    54.0,
                    if submit_hover {
                        Color::new(0.20, 0.65, 0.86, 1.0)
                    } else {
                        Color::new(0.13, 0.49, 0.72, 1.0)
                    },
                );
                draw_rectangle_lines(180.0, 752.0, 360.0, 54.0, 1.5, WHITE);
                draw_centered("SAVE SCORE & RETURN HOME", cx, 787.0, 19.0, WHITE, f);
                sector_submit = submit_hover && ui_tap;
                if sector_submit {
                    require_touch_release = true;
                    button_lock_timer = BUTTON_LOCK_DELAY;
                }

                // Next Sector button
                let nx_y = VIRTUAL_HEIGHT * 0.65;
                let btn_avail = current_sector + 1 < sectors.len();
                let nx_label = if btn_avail {
                    "NEXT SECTOR"
                } else {
                    "GALAXY COMPLETED"
                };
                let nxh = hit(ptr, cx - 150.0, nx_y, 300.0, 66.0);
                draw_rectangle(
                    cx - 150.0,
                    nx_y,
                    300.0,
                    66.0,
                    if nxh {
                        Color::new(0.28, 0.85, 0.45, 1.0)
                    } else {
                        Color::new(0.18, 0.68, 0.34, 1.0)
                    },
                );
                draw_rectangle_lines(cx - 150.0, nx_y, 300.0, 66.0, 2.0, WHITE);
                draw_centered(nx_label, cx, nx_y + 42.0, 26.0, WHITE, f);

                // Galaxy Map button
                let mp_y = nx_y + 84.0;
                let mph = hit(ptr, cx - 150.0, mp_y, 300.0, 60.0);
                draw_rectangle(
                    cx - 150.0,
                    mp_y,
                    300.0,
                    60.0,
                    if mph {
                        Color::new(0.42, 0.22, 0.72, 1.0)
                    } else {
                        Color::new(0.30, 0.15, 0.55, 1.0)
                    },
                );
                draw_rectangle_lines(cx - 150.0, mp_y, 300.0, 60.0, 1.8, WHITE);
                draw_centered("GALAXY MAP", cx, mp_y + 38.0, 22.0, WHITE, f);
            }

            // ── Paused ───────────────────────────────────────────────────────
            GameState::Paused => {
                for body in &bodies {
                    BodyRenderer::draw_body(body);
                }
                particles.draw();

                // Frosted backdrop
                draw_rectangle(
                    0.0,
                    0.0,
                    VIRTUAL_WIDTH,
                    VIRTUAL_HEIGHT,
                    Color::new(0.02, 0.03, 0.09, 0.82),
                );
                let cx = VIRTUAL_WIDTH * 0.5;

                // Centered Frosted Modal Card
                let card_w = 460.0;
                let card_h = 420.0;
                let card_x = cx - card_w * 0.5;
                let card_y = VIRTUAL_HEIGHT * 0.26;

                // Outer soft halo
                draw_rectangle(card_x - 3.0, card_y - 3.0, card_w + 6.0, card_h + 6.0, Color::new(0.18, 0.35, 0.70, 0.25));
                // Card body
                draw_rectangle(card_x, card_y, card_w, card_h, Color::new(0.06, 0.08, 0.22, 0.95));
                // Glowing border
                draw_rectangle_lines(card_x, card_y, card_w, card_h, 2.0, Color::new(0.38, 0.75, 1.0, 0.85));
                draw_rectangle_lines(card_x + 4.0, card_y + 4.0, card_w - 8.0, card_h - 8.0, 1.0, Color::new(0.20, 0.38, 0.75, 0.35));

                // Header
                draw_centered("⏸  MISSION PAUSED", cx, card_y + 48.0, 36.0, WHITE, f);
                draw_centered("ORBITAL STABILIZERS ENGAGED", cx, card_y + 78.0, 16.0, Color::new(0.45, 0.85, 1.0, 0.85), f);
                draw_line(card_x + 30.0, card_y + 96.0, card_x + card_w - 30.0, card_y + 96.0, 1.0, Color::new(0.30, 0.50, 0.85, 0.40));

                // Run stats readout
                let stat_box_w = 400.0;
                let stat_box_h = 76.0;
                let stat_box_x = cx - stat_box_w * 0.5;
                let stat_box_y = card_y + 114.0;
                draw_rectangle(stat_box_x, stat_box_y, stat_box_w, stat_box_h, Color::new(0.09, 0.12, 0.30, 0.75));
                draw_rectangle_lines(stat_box_x, stat_box_y, stat_box_w, stat_box_h, 1.2, Color::new(0.25, 0.50, 0.85, 0.50));

                let score_lbl = format!("SCORE: {}", current_score);
                draw_centered(&score_lbl, cx, stat_box_y + 32.0, 24.0, Color::new(1.0, 0.88, 0.35, 1.0), f);

                let dust_lbl = format!("STARDUST EARNED:  +{} ✨", run_stardust);
                draw_centered(&dust_lbl, cx, stat_box_y + 60.0, 18.0, Color::new(0.55, 0.90, 1.0, 0.90), f);

                // ── Interactive Buttons ──
                let bw = 380.0;
                let bx = cx - bw * 0.5;
                let ry = card_y + 242.0;
                let rh = hit(ptr, bx, ry, bw, 66.0);

                // 1. RESUME FLIGHT (Emerald/Cyan)
                if rh {
                    draw_rectangle(bx - 3.0, ry - 3.0, bw + 6.0, 72.0, Color::new(0.20, 0.82, 0.50, 0.35));
                }
                draw_rectangle(
                    bx,
                    ry,
                    bw,
                    66.0,
                    if rh {
                        Color::new(0.18, 0.76, 0.46, 0.98)
                    } else {
                        Color::new(0.11, 0.60, 0.36, 0.92)
                    },
                );
                draw_rectangle_lines(bx, ry, bw, 66.0, 2.0, Color::new(0.45, 1.0, 0.70, 0.95));
                draw_line(bx + 16.0, ry + 3.0, bx + bw - 16.0, ry + 3.0, 1.5, Color::new(0.65, 1.0, 0.82, 0.65));
                draw_vector_play(bx + 40.0, ry + 33.0, 24.0, WHITE);
                draw_centered("RESUME FLIGHT", cx + 12.0, ry + 44.0, 26.0, WHITE, f);

                // 2. END RUN (Ruby/Coral)
                let qy = ry + 80.0;
                let qh = hit(ptr, bx, qy, bw, 66.0);
                if qh {
                    draw_rectangle(bx - 3.0, qy - 3.0, bw + 6.0, 72.0, Color::new(0.85, 0.25, 0.35, 0.35));
                }
                draw_rectangle(
                    bx,
                    qy,
                    bw,
                    66.0,
                    if qh {
                        Color::new(0.78, 0.22, 0.30, 0.98)
                    } else {
                        Color::new(0.58, 0.15, 0.22, 0.92)
                    },
                );
                draw_rectangle_lines(bx, qy, bw, 66.0, 1.8, Color::new(1.0, 0.45, 0.55, 0.90));
                draw_line(bx + 16.0, qy + 3.0, bx + bw - 16.0, qy + 3.0, 1.5, Color::new(1.0, 0.70, 0.75, 0.55));
                let end_label = if current_score > 0 { "END RUN & SUBMIT SCORE" } else { "ABANDON RUN" };
                draw_centered(end_label, cx, qy + 44.0, 24.0, WHITE, f);
            }

            // ── GameOver ─────────────────────────────────────────────────────
            GameState::GameOver => {
                for body in &bodies {
                    BodyRenderer::draw_body(body);
                }
                particles.draw();
                go_action = GameOverModal::draw(
                    current_score,
                    save_data.high_score,
                    run_stardust,
                    revives_used < FREE_REVIVES_PER_RUN,
                    stardust_doubled,
                    ads.is_rewarded_ready(),
                    &ad_status,
                    &save_data.public_name,
                    &leaderboard_status,
                    ptr,
                    ui_tap,
                    f,
                );
                if go_action != GameOverAction::None {
                    require_touch_release = true;
                    button_lock_timer = BUTTON_LOCK_DELAY;
                }
                if web_submit {
                    go_action = GameOverAction::SaveScore;
                }
            }

            // ── Shop ─────────────────────────────────────────────────────────
            GameState::Shop => {
                shop_action = ShopModal::draw(
                    &catalog,
                    save_data.stardust,
                    billing.is_ad_removed(),
                    &save_data.equipped_skin,
                    &save_data.unlocked_skins,
                    shop_msg.as_deref(),
                    ptr,
                    ui_tap,
                    f,
                );
                if !matches!(shop_action, ShopAction::None) {
                    require_touch_release = true;
                    button_lock_timer = BUTTON_LOCK_DELAY;
                }
            }

            // ── Settings ─────────────────────────────────────────────────────
            GameState::Settings => {
                let action = SettingsModal::draw(
                    save_data.resolution_profile,
                    save_data.sound_enabled,
                    save_data.haptics_enabled,
                    ptr,
                    ui_tap,
                    f,
                );
                match action {
                    SettingsAction::ChangeResolution(prof) => {
                        save_data.resolution_profile = prof;
                        let (rw, rh) = prof.dimensions();
                        target_w = rw;
                        target_h = rh;
                        vt = render_target(target_w, target_h);
                        vt.texture.set_filter(FilterMode::Linear);
                        vcam = Camera2D {
                            target: vec2(VIRTUAL_WIDTH * 0.5, VIRTUAL_HEIGHT * 0.5),
                            zoom: vec2(2.0 / VIRTUAL_WIDTH, 2.0 / VIRTUAL_HEIGHT),
                            offset: vec2(0.0, 0.0),
                            rotation: 0.0,
                            render_target: Some(vt.clone()),
                            viewport: None,
                        };
                        let _ = save_mgr.save(&save_data);
                    }
                    SettingsAction::ToggleSound => {
                        save_data.sound_enabled = !save_data.sound_enabled;
                        let _ = save_mgr.save(&save_data);
                    }
                    SettingsAction::ToggleHaptics => {
                        save_data.haptics_enabled = !save_data.haptics_enabled;
                        let _ = save_mgr.save(&save_data);
                    }
                    SettingsAction::Close => {
                        game_state = shop_return_state;
                    }
                    SettingsAction::None => {}
                }
            }

            // ── Achievements ─────────────────────────────────────────────────
            GameState::Achievements => {
                let action = AchievementsModal::draw(
                    &save_data.unlocked_achievements,
                    current_score.max(save_data.high_score),
                    ptr,
                    ui_tap,
                    f,
                );
                if matches!(action, AchievementsAction::Close) {
                    game_state = shop_return_state;
                }
            }

            // ── Ad Overlay ───────────────────────────────────────────────────
            GameState::WatchingAd => {
                AdOverlay::draw(f);
            }
        }

        // ── Resolve Modal Actions ────────────────────────────────────────────
        match go_action {
            GameOverAction::WatchAdRevive => {
                require_touch_release = true;
                button_lock_timer = BUTTON_LOCK_DELAY;
                if ads.start_rewarded_ad(RewardType::EventHorizonRevive) {
                    ad_status.clear();
                    game_state = GameState::WatchingAd;
                } else {
                    ad_status = "Ad unavailable. Please try again later.".to_string();
                }
            }
            GameOverAction::WatchAdDoubleStardust => {
                require_touch_release = true;
                button_lock_timer = BUTTON_LOCK_DELAY;
                if ads.start_rewarded_ad(RewardType::DoubleStardust) {
                    ad_status.clear();
                    game_state = GameState::WatchingAd;
                } else {
                    ad_status = "Ad unavailable. Please try again later.".to_string();
                }
            }
            GameOverAction::Restart => {
                require_touch_release = true;
                button_lock_timer = BUTTON_LOCK_DELAY;
                shop_return_state = GameState::MainMenu;
                ad_status.clear();
                let sec = &sectors[current_sector];
                objective = ObjectiveTracker::new(sec.objective.clone());
                bodies.clear();
                next_body_id = 1;
                current_score = 0;
                run_stardust = 0;
                revives_used = 0;
                stardust_doubled = false;
                danger_timer = 0.0;
                ability_charges = 0;
                super_flare_charges = 0;
                super_flare_milestone = 0;
                gravity_wave_grace_timer = 0.0;
                drop_cooldown = 0.0;
                drop_pool = DropPool::default();
                next_tier = drop_pool.random_tier();
                is_aiming = false;
                game_state = GameState::Playing;
            }
            GameOverAction::OpenShop => {
                require_touch_release = true;
                button_lock_timer = BUTTON_LOCK_DELAY;
                shop_return_state = GameState::GameOver;
                game_state = GameState::Shop;
            }
            GameOverAction::OpenAchievements => {
                require_touch_release = true;
                button_lock_timer = BUTTON_LOCK_DELAY;
                shop_return_state = GameState::GameOver;
                game_state = GameState::Achievements;
            }
            GameOverAction::SaveScore => {
                require_touch_release = true;
                button_lock_timer = BUTTON_LOCK_DELAY;
                let name_trim = save_data.public_name.trim();
                if name_trim.is_empty() {
                    current_score = 0;
                    run_stardust = 0;
                    shop_return_state = GameState::MainMenu;
                    game_state = GameState::MainMenu;
                } else {
                    match validate_public_name(&save_data.public_name) {
                        Err(message) => leaderboard_status = message.to_string(),
                        Ok(name) => match save_mgr.save(&save_data) {
                            Err(_) => {
                                leaderboard_status =
                                    "Could not save. Check storage access and try again.".to_string();
                            }
                            Ok(()) => {
                                leaderboard.submit(&name, current_score).await;
                                leaderboard_status = leaderboard.status.clone();
                                current_score = 0;
                                run_stardust = 0;
                                shop_return_state = GameState::MainMenu;
                                game_state = GameState::MainMenu;
                            }
                        },
                    }
                }
            }
            GameOverAction::None => {}
        }

        if sector_submit || (web_submit && game_state == GameState::SectorComplete) {
            require_touch_release = true;
            button_lock_timer = BUTTON_LOCK_DELAY;
            match validate_public_name(&save_data.public_name) {
                Err(message) => leaderboard_status = message.to_string(),
                Ok(name) => match save_mgr.save(&save_data) {
                    Err(_) => {
                        leaderboard_status =
                            "Could not save. Check storage access and try again.".to_string();
                    }
                    Ok(()) => {
                        leaderboard.submit(&name, current_score).await;
                        leaderboard_status = leaderboard.status.clone();
                        current_score = 0;
                        run_stardust = 0;
                        shop_return_state = GameState::MainMenu;
                        game_state = GameState::MainMenu;
                    }
                },
            }
        }

        match shop_action {
            ShopAction::BuyItem(ref sku) => {
                require_touch_release = true;
                button_lock_timer = BUTTON_LOCK_DELAY;
                if sku.contains("removeads") {
                    billing.purchase_product(sku).ok();
                    save_data.ads_removed = true;
                } else if sku.contains("starpass") {
                    billing.purchase_product(sku).ok();
                    save_data.celestial_pass_unlocked = true;
                } else if sku.ends_with(".500") {
                    save_data.stardust += 500;
                } else if sku.ends_with(".2000") {
                    save_data.stardust += 2000;
                } else if let Some(item) = catalog.skin_items.iter().find(|i| i.id == *sku) {
                    if save_data.stardust >= item.stardust_price
                        && !save_data.unlocked_skins.contains(sku)
                    {
                        save_data.stardust -= item.stardust_price;
                        save_data.unlocked_skins.push(sku.clone());
                        save_data.equipped_skin = sku.clone();
                        shop_msg = Some(format!("Equipped {}!", item.title));
                    }
                }
                let _ = save_mgr.save(&save_data);
            }
            ShopAction::EquipSkin(ref sid) => {
                require_touch_release = true;
                button_lock_timer = BUTTON_LOCK_DELAY;
                save_data.equipped_skin = sid.clone();
                let _ = save_mgr.save(&save_data);
            }
            ShopAction::Close => {
                shop_msg = None;
                require_touch_release = true;
                button_lock_timer = BUTTON_LOCK_DELAY;
                game_state = shop_return_state;
            }
            ShopAction::None => {}
        }

        // Lock all buttons if state changed or any action occurred this frame
        if game_state != state_before_frame
            || go_action != GameOverAction::None
            || !matches!(shop_action, ShopAction::None)
            || sector_submit
        {
            require_touch_release = true;
            button_lock_timer = BUTTON_LOCK_DELAY;
        }

        // ── Blit High-Resolution Target to Physical Screen ───────────────────
        set_default_camera();
        clear_background(Color::new(0.012, 0.012, 0.03, 1.0));
        draw_texture_ex(
            &vt.texture,
            ox,
            oy,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(VIRTUAL_WIDTH * scale, VIRTUAL_HEIGHT * scale)),
                flip_y: false,
                ..Default::default()
            },
        );

        next_frame().await;
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Reward handler
// ─────────────────────────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
pub fn handle_reward(
    reward: RewardType,
    _score: &mut u64,
    run_stardust: &mut u64,
    revives_used: &mut u32,
    stardust_doubled: &mut bool,
    bodies: &mut Vec<CelestialBody>,
    danger_timer: &mut f32,
    save_data: &mut SaveData,
    save_mgr: &SaveManager,
    game_state: &mut GameState,
) {
    match reward {
        RewardType::EventHorizonRevive => {
            *revives_used += 1;
            *danger_timer = 0.0;
            // Clear bodies above danger line
            bodies.retain(|b| b.pos.y > JAR_TOP_LINE + b.radius);
            *game_state = GameState::Playing;
        }
        RewardType::DoubleStardust => {
            *stardust_doubled = true;
            save_data.stardust += *run_stardust;
            *run_stardust *= 2;
            let _ = save_mgr.save(save_data);
            *game_state = GameState::GameOver;
        }
        RewardType::DailyChest => {
            save_data.stardust += 250;
            let _ = save_mgr.save(save_data);
            *game_state = GameState::MainMenu;
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Native Entry Points for All Ecosystems (miniquad & Android)
// ─────────────────────────────────────────────────────────────────────────────

#[no_mangle]
pub extern "C" fn quad_main() {
    // Called by miniquad on Android and desktop runtimes
    macroquad::Window::from_config(window_conf(), game_main());
}

#[cfg(target_os = "android")]
#[no_mangle]
pub extern "C" fn android_main(_app: *mut std::ffi::c_void) {
    macroquad::Window::from_config(window_conf(), game_main());
}
