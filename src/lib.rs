pub mod core;
pub mod physics;
pub mod graphics;
pub mod audio;
pub mod monetization;
pub mod ui;

use macroquad::prelude::*;
use crate::core::config::*;
use crate::core::game_state::*;
use crate::core::save_system::{SaveData, SaveManager};
use crate::core::sector::{ObjectiveTracker, all_sectors, rate_run};
use crate::physics::*;
use crate::graphics::*;
use crate::audio::AudioEngine;
use crate::monetization::ads::{AdService, MockAdService, RewardType};
use crate::monetization::billing::{BillingService, MockBillingService};
use crate::monetization::economy::EconomyCatalog;
use crate::ui::*;

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
    draw_text_ex(text, cx - dim.width * 0.5, y, TextParams { font, font_size: sz, color, ..Default::default() });
}

pub fn draw_txt(text: &str, x: f32, y: f32, size: f32, color: Color, font: Option<&Font>) {
    draw_text_ex(text, x, y, TextParams { font, font_size: size as u16, color, ..Default::default() });
}

pub fn hit(pos: Vec2, x: f32, y: f32, w: f32, h: f32) -> bool {
    pos.x >= x && pos.x <= x + w && pos.y >= y && pos.y <= y + h
}

// ─────────────────────────────────────────────────────────────────────────────
//  Strategic Abilities
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ability {
    GravityWave,   // Nudge all bodies toward jar centre & upwards to unblock merges
    SolarFlare,    // Vaporise the 2 highest clutter bodies
}

impl Ability {
    pub fn label(self) -> &'static str {
        match self {
            Self::GravityWave => "GRAVITY WAVE",
            Self::SolarFlare  => "SOLAR FLARE",
        }
    }
    pub fn description(self) -> &'static str {
        match self {
            Self::GravityWave => "Pull & lift bodies to center",
            Self::SolarFlare  => "Vaporise top 2 clutter bodies",
        }
    }
}

fn apply_ability(ability: Ability, bodies: &mut Vec<CelestialBody>, particles: &mut ParticleEngine) {
    match ability {
        Ability::GravityWave => {
            let cx = (JAR_LEFT + JAR_RIGHT) * 0.5;
            for b in bodies.iter_mut() {
                let dx = cx - b.pos.x;
                b.vel.x += dx * 0.40;
                b.vel.y -= 160.0; // gentle upward impulse to allow trapped bodies to merge
                particles.spawn_trail(b.pos, Color::new(0.35, 0.75, 1.0, 0.8));
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
            let to_remove: Vec<u64> = indices
                .iter()
                .take(2)
                .map(|(i, _)| bodies[*i].id)
                .collect();

            for b in bodies.iter() {
                if to_remove.contains(&b.id) {
                    particles.spawn_fusion_burst(b.pos, Color::new(1.0, 0.5, 0.2, 1.0), 5);
                }
            }
            bodies.retain(|b| !to_remove.contains(&b.id));
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Main Game Loop
// ─────────────────────────────────────────────────────────────────────────────

pub async fn game_main() {
    // ── Font ─────────────────────────────────────────────────────────────────
    let font_path = if cfg!(target_os = "android") { "font.ttf" } else { "assets/font.ttf" };
    let font: Option<Font> = load_ttf_font(font_path).await.ok();
    let f: Option<&Font> = font.as_ref();

    // ── Core Systems ─────────────────────────────────────────────────────────
    let save_mgr = SaveManager::new();
    let mut save_data = save_mgr.load();
    let audio = AudioEngine::new().await;
    let mut starfield = Starfield::new();
    let mut particles = ParticleEngine::new();
    let mut ads = MockAdService::new();
    let mut billing = MockBillingService::new(save_data.ads_removed, save_data.celestial_pass_unlocked);
    let catalog = EconomyCatalog::new();
    let sectors = all_sectors();

    // ── Virtual Render Target (720 × 1280) ───────────────────────────────────
    let vt = render_target(VIRTUAL_WIDTH as u32, VIRTUAL_HEIGHT as u32);
    vt.texture.set_filter(FilterMode::Linear);
    let mut vcam = Camera2D::from_display_rect(Rect::new(0.0, 0.0, VIRTUAL_WIDTH, VIRTUAL_HEIGHT));
    vcam.render_target = Some(vt.clone());

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

    // Objective tracker
    let mut objective = ObjectiveTracker::new(sectors[0].objective.clone());

    // Danger timer (settled bodies overflowing above JAR_TOP_LINE)
    let mut danger_timer = 0.0f32;

    // Two-body choice + Comet Reserve
    let mut choice_a = CelestialTier::random_spawn_tier();
    let mut choice_b = CelestialTier::random_spawn_tier();
    let mut selected_choice = 0usize; // 0 = A, 1 = B
    let mut reserve: Option<CelestialTier> = None; // Comet Reserve slot
    let mut drop_x = VIRTUAL_WIDTH * 0.5;
    let mut is_aiming = false;
    let mut drop_cooldown = 0.0f32;

    // Strategic abilities
    let mut ability_charges: u32 = 0;
    let mut ability_flash: f32 = 0.0;

    // Sector completion celebration
    let mut sector_complete_timer = 0.0f32;
    let mut earned_stars = 0u8;

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
        let ptr = if let Some(t) = touch_list.first() {
            Vec2::new((t.position.x - ox) / scale, (t.position.y - oy) / scale)
        } else {
            vm
        };

        let mouse_pressed = is_mouse_button_pressed(MouseButton::Left);
        let mouse_down = is_mouse_button_down(MouseButton::Left);
        let mouse_released = is_mouse_button_released(MouseButton::Left);

        let touch_started = touch_list.iter().any(|t| t.phase == TouchPhase::Started);
        let touch_ended = touch_list.iter().any(|t| t.phase == TouchPhase::Ended);
        let touch_down = !touch_list.is_empty();

        let tap = mouse_pressed || touch_started;
        let is_held = mouse_down || touch_down;
        let just_released = mouse_released || touch_ended;

        // ── Global Ticks ─────────────────────────────────────────────────────
        starfield.update(dt);
        particles.update(dt);
        if ability_flash > 0.0 { ability_flash -= dt; }
        if let Some(reward) = ads.update(dt) {
            handle_reward(reward, &mut current_score, &mut run_stardust, &mut revives_used,
                &mut stardust_doubled, &mut bodies, &mut danger_timer,
                &mut save_data, &save_mgr, &mut game_state);
        }

        // ── LOGIC ────────────────────────────────────────────────────────────
        match game_state {

            // ── Main Menu ────────────────────────────────────────────────────
            GameState::MainMenu => {
                let cx = VIRTUAL_WIDTH * 0.5;
                let bw = 340.0;
                let bx = cx - bw * 0.5;
                let play_y = VIRTUAL_HEIGHT * 0.58;
                let shop_y = play_y + 85.0;

                if (tap && hit(ptr, bx, play_y, bw, 68.0)) || is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Enter) {
                    game_state = GameState::GalaxyMap;
                }
                if tap && hit(ptr, bx, shop_y, bw, 62.0) {
                    game_state = GameState::Shop;
                }
            }

            // ── Galaxy Map (3 Chapter Tabs) ──────────────────────────────────
            GameState::GalaxyMap => {
                // Back button (top left)
                if (tap && hit(ptr, 20.0, 20.0, 90.0, 50.0)) || is_key_pressed(KeyCode::Escape) {
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
                    if tap && hit(ptr, tx, tab_y, tab_w, tab_h) {
                        selected_chapter = c;
                    }
                }

                // Keyboard chapter switching (Q / E or 1 / 2 / 3)
                if is_key_pressed(KeyCode::Key1) { selected_chapter = 0; }
                if is_key_pressed(KeyCode::Key2) { selected_chapter = 1; }
                if is_key_pressed(KeyCode::Key3) { selected_chapter = 2; }

                // Sector cards in current chapter (5 cards per chapter)
                let start_idx = selected_chapter * 5;
                for row in 0..5 {
                    let idx = start_idx + row;
                    if idx >= sectors.len() { break; }
                    let card_y = 195.0 + row as f32 * 115.0;
                    let card_x = 40.0;
                    let card_w = VIRTUAL_WIDTH - 80.0;
                    let card_h = 100.0;

                    if save_data.is_sector_unlocked(idx) && tap && hit(ptr, card_x, card_y, card_w, card_h) {
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
                        choice_a = CelestialTier::random_spawn_tier();
                        choice_b = CelestialTier::random_spawn_tier();
                        selected_choice = 0;
                        reserve = None;
                        is_aiming = false;
                        game_state = GameState::Playing;
                        break;
                    }
                }
            }

            // ── Playing ──────────────────────────────────────────────────────
            GameState::Playing => {
                let active_tier = if selected_choice == 0 { choice_a } else { choice_b };
                let r = active_tier.radius();

                // Drop cooldown timer
                if drop_cooldown > 0.0 { drop_cooldown -= dt; }

                // Pause button (top right: x: 640..700, y: 20..68)
                if (tap && hit(ptr, VIRTUAL_WIDTH - 80.0, 20.0, 60.0, 48.0)) || is_key_pressed(KeyCode::P) || is_key_pressed(KeyCode::Escape) {
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

                // Choice Selection UI (y: 195..280)
                let ch_y = 195.0;
                let ch_w = 180.0;
                let ch_h = 80.0;

                let mut clicked_ui = false;

                if tap {
                    // Choice A button
                    if hit(ptr, JAR_LEFT, ch_y, ch_w, ch_h) {
                        selected_choice = 0;
                        clicked_ui = true;
                    }
                    // Choice B button
                    let b_x = JAR_RIGHT - ch_w;
                    if hit(ptr, b_x, ch_y, ch_w, ch_h) {
                        selected_choice = 1;
                        clicked_ui = true;
                    }
                    // Reserve slot (Center circle)
                    let res_cx = VIRTUAL_WIDTH * 0.5;
                    let res_cy = ch_y + 40.0;
                    if (ptr - Vec2::new(res_cx, res_cy)).length() <= 40.0 {
                        let current = if selected_choice == 0 { choice_a } else { choice_b };
                        if let Some(res) = reserve {
                            if selected_choice == 0 { choice_a = res; } else { choice_b = res; }
                            reserve = Some(current);
                        } else {
                            reserve = Some(current);
                            if selected_choice == 0 { choice_a = CelestialTier::random_spawn_tier(); }
                            else { choice_b = CelestialTier::random_spawn_tier(); }
                        }
                        clicked_ui = true;
                    }

                    // Strategic Abilities (strictly below jar floor, y: 1140..1220)
                    let ab_y = 1140.0;
                    let ab_w = 250.0;
                    let ab_h = 70.0;
                    if ability_charges > 0 {
                        // Wave button
                        if hit(ptr, JAR_LEFT, ab_y, ab_w, ab_h) {
                            apply_ability(Ability::GravityWave, &mut bodies, &mut particles);
                            audio.play_fusion_chime(4);
                            ability_charges -= 1;
                            clicked_ui = true;
                        }
                        // Flare button
                        let flare_x = JAR_RIGHT - ab_w;
                        if hit(ptr, flare_x, ab_y, ab_w, ab_h) {
                            apply_ability(Ability::SolarFlare, &mut bodies, &mut particles);
                            audio.play_fusion_chime(6);
                            ability_charges -= 1;
                            clicked_ui = true;
                        }
                    }
                }

                // Keyboard shortcuts for Reserve and Abilities
                if is_key_pressed(KeyCode::R) || is_key_pressed(KeyCode::B) {
                    let current = if selected_choice == 0 { choice_a } else { choice_b };
                    if let Some(res) = reserve {
                        if selected_choice == 0 { choice_a = res; } else { choice_b = res; }
                        reserve = Some(current);
                    } else {
                        reserve = Some(current);
                        if selected_choice == 0 { choice_a = CelestialTier::random_spawn_tier(); }
                        else { choice_b = CelestialTier::random_spawn_tier(); }
                    }
                }
                if ability_charges > 0 {
                    if is_key_pressed(KeyCode::Key1) {
                        apply_ability(Ability::GravityWave, &mut bodies, &mut particles);
                        audio.play_fusion_chime(4);
                        ability_charges -= 1;
                    } else if is_key_pressed(KeyCode::Key2) {
                        apply_ability(Ability::SolarFlare, &mut bodies, &mut particles);
                        audio.play_fusion_chime(6);
                        ability_charges -= 1;
                    }
                }

                // Touch / Mouse Aiming inside Jar Area
                let in_aim_zone = ptr.x >= JAR_LEFT - 20.0 && ptr.x <= JAR_RIGHT + 20.0
                    && ptr.y >= DROP_Y - 20.0 && ptr.y <= JAR_BOTTOM;

                if is_held && in_aim_zone && !clicked_ui {
                    drop_x = ptr.x.clamp(JAR_LEFT + r, JAR_RIGHT - r);
                    is_aiming = true;
                }

                // Commit Drop on Release or Spacebar
                let keyboard_drop = is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Down);
                let commit_drop = (just_released && is_aiming && !clicked_ui) || keyboard_drop;

                if commit_drop && drop_cooldown <= 0.0 {
                    let tier = if selected_choice == 0 { choice_a } else { choice_b };
                    let body = CelestialBody::new(next_body_id, tier, Vec2::new(drop_x, DROP_Y), Vec2::new(0.0, 140.0));
                    next_body_id += 1;
                    bodies.push(body);
                    audio.play_slingshot();
                    particles.spawn_trail(Vec2::new(drop_x, DROP_Y), tier.primary_color());

                    // Advance spawner choices
                    if selected_choice == 0 { choice_a = CelestialTier::random_spawn_tier(); }
                    else { choice_b = CelestialTier::random_spawn_tier(); }
                    drop_cooldown = DROP_COOLDOWN;
                    is_aiming = false;
                }

                // Physics Simulation Step
                for b in bodies.iter_mut() { b.update(dt); }
                let fusions = CollisionEngine::resolve_collisions(&mut bodies);
                for fus in fusions {
                    current_score += fus.score_awarded;
                    run_stardust += fus.stardust_awarded as u64;

                    // Milestone merges grant Ability Charges
                    if (fus.new_tier as usize) >= ABILITY_EARN_TIER && ability_charges < MAX_ABILITY_CHARGES {
                        ability_charges += 1;
                        ability_flash = 0.9;
                    }

                    objective.on_merge(fus.new_tier);
                    objective.score_progress = current_score;
                    particles.spawn_fusion_burst(fus.pos, fus.new_tier.primary_color(), fus.new_tier as usize);
                    let pts_label = format!("+{}", fus.score_awarded);
                    particles.add_floating_text(pts_label, fus.pos, fus.new_tier.glow_color(), 28.0);
                    audio.play_fusion_chime(fus.new_tier as usize);

                    if (fus.new_tier as usize) >= 5 {
                        let name = format!("{} FORMED!", fus.new_tier.name().to_uppercase());
                        particles.add_floating_text(name, fus.pos - Vec2::new(0.0, 50.0), fus.new_tier.label_color(), 34.0);
                    }
                }
                objective.score_progress = current_score;

                // Live high score tracking
                if current_score > save_data.high_score {
                    save_data.high_score = current_score;
                }

                // Sector Objective Completion Check
                if objective.is_complete() && game_state == GameState::Playing {
                    earned_stars = rate_run(current_sector, current_score);
                    save_data.complete_sector(current_sector, earned_stars);
                    save_data.stardust += run_stardust;
                    let _ = save_mgr.save(&save_data);
                    sector_complete_timer = 0.0;
                    audio.play_victory();
                    game_state = GameState::SectorComplete;
                }

                // Danger Overflow Timer
                let any_danger = bodies.iter().any(|b| b.above_danger_line());
                if any_danger {
                    danger_timer += dt;
                    if danger_timer >= CRITICAL_TIME_LIMIT {
                        audio.play_game_over();
                        save_data.runs_played += 1;
                        save_data.stardust += run_stardust;
                        let _ = save_mgr.save(&save_data);
                        if !billing.is_ad_removed() && save_data.runs_played.is_multiple_of(INTERSTITIAL_RUN_INTERVAL) {
                            ads.start_interstitial_ad();
                            game_state = GameState::WatchingAd;
                        } else {
                            game_state = GameState::GameOver;
                        }
                    }
                } else if danger_timer > 0.0 {
                    danger_timer = (danger_timer - dt * 2.0).max(0.0);
                }
            }

            // ── Sector Complete ──────────────────────────────────────────────
            GameState::SectorComplete => {
                sector_complete_timer += dt;
                let cx = VIRTUAL_WIDTH * 0.5;
                let nx_y = VIRTUAL_HEIGHT * 0.65;
                let btn_avail = current_sector + 1 < sectors.len();

                // Next Sector button
                if btn_avail && (tap && hit(ptr, cx - 150.0, nx_y, 300.0, 66.0) || is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Enter)) {
                    current_sector += 1;
                    let sec = &sectors[current_sector];
                    objective = ObjectiveTracker::new(sec.objective.clone());
                    bodies.clear(); next_body_id = 1; current_score = 0; run_stardust = 0;
                    revives_used = 0; stardust_doubled = false; danger_timer = 0.0;
                    ability_charges = 0; drop_cooldown = 0.0;
                    choice_a = CelestialTier::random_spawn_tier(); choice_b = CelestialTier::random_spawn_tier();
                    selected_choice = 0; reserve = None; is_aiming = false;
                    game_state = GameState::Playing;
                }

                // Galaxy Map button
                let mp_y = nx_y + 84.0;
                if tap && hit(ptr, cx - 150.0, mp_y, 300.0, 60.0) {
                    game_state = GameState::GalaxyMap;
                }
            }

            // ── Paused ───────────────────────────────────────────────────────
            GameState::Paused => {
                let cx = VIRTUAL_WIDTH * 0.5;
                let ry = VIRTUAL_HEIGHT * 0.42;
                if (tap && hit(ptr, cx - 140.0, ry, 280.0, 64.0)) || is_key_pressed(KeyCode::P) || is_key_pressed(KeyCode::Escape) || is_key_pressed(KeyCode::Space) {
                    game_state = GameState::Playing;
                }
                if tap && hit(ptr, cx - 140.0, ry + 84.0, 280.0, 64.0) {
                    game_state = GameState::GalaxyMap;
                }
            }

            _ => {}
        }

        // ── RENDER ────────────────────────────────────────────────────────────
        set_camera(&vcam);
        clear_background(Color::new(0.02, 0.02, 0.06, 1.0));

        let danger_ratio = (danger_timer / CRITICAL_TIME_LIMIT).clamp(0.0, 1.0);
        starfield.draw(danger_ratio);

        let mut go_action = GameOverAction::None;
        let mut shop_action = ShopAction::None;
        let mut ad_result = MockAdResult::None;

        match game_state {

            // ── Main Menu ────────────────────────────────────────────────────
            GameState::MainMenu => {
                let cx = VIRTUAL_WIDTH * 0.5;

                // Title Banner
                draw_rectangle(40.0, VIRTUAL_HEIGHT * 0.16, VIRTUAL_WIDTH - 80.0, 220.0, Color::new(0.05, 0.04, 0.14, 0.85));
                draw_rectangle_lines(40.0, VIRTUAL_HEIGHT * 0.16, VIRTUAL_WIDTH - 80.0, 220.0, 1.8, Color::new(0.45, 0.35, 0.80, 0.55));
                draw_centered("GRAVIPOP", cx, VIRTUAL_HEIGHT * 0.28, 76.0, WHITE, f);
                draw_centered("STELLAR CONSERVATORY", cx, VIRTUAL_HEIGHT * 0.28 + 52.0, 26.0, Color::new(0.40, 0.85, 1.0, 0.95), f);
                draw_centered("Restore the Persistent Galaxy", cx, VIRTUAL_HEIGHT * 0.28 + 92.0, 20.0, Color::new(0.70, 0.70, 0.85, 0.80), f);

                let hi_txt = format!("BEST SCORE  {}", save_data.high_score);
                draw_centered(&hi_txt, cx, VIRTUAL_HEIGHT * 0.28 + 128.0, 22.0, Color::new(1.0, 0.85, 0.30, 1.0), f);

                // Explore Galaxy Button
                let bw = 340.0;
                let bx = cx - bw * 0.5;
                let play_y = VIRTUAL_HEIGHT * 0.58;
                let play_hov = hit(ptr, bx, play_y, bw, 68.0);
                draw_rectangle(bx, play_y, bw, 68.0, if play_hov { Color::new(0.28, 0.88, 0.48, 1.0) } else { Color::new(0.18, 0.70, 0.34, 1.0) });
                draw_rectangle_lines(bx, play_y, bw, 68.0, 2.0, WHITE);
                draw_vector_play(bx + 40.0, play_y + 34.0, 24.0, WHITE);
                draw_centered("EXPLORE GALAXY", cx + 12.0, play_y + 44.0, 26.0, WHITE, f);

                // Cosmic Shop Button
                let shop_y = play_y + 85.0;
                let shop_hov = hit(ptr, bx, shop_y, bw, 62.0);
                draw_rectangle(bx, shop_y, bw, 62.0, if shop_hov { Color::new(0.55, 0.28, 0.86, 1.0) } else { Color::new(0.38, 0.20, 0.66, 1.0) });
                draw_rectangle_lines(bx, shop_y, bw, 62.0, 1.8, WHITE);
                draw_vector_gem(bx + 40.0, shop_y + 31.0, 22.0, Color::new(0.40, 0.90, 1.0, 1.0));
                draw_centered("COSMIC STORE", cx + 12.0, shop_y + 40.0, 24.0, WHITE, f);

                // Stardust Badge at Bottom
                let dust_cx = cx;
                let dust_y = VIRTUAL_HEIGHT - 60.0;
                draw_vector_gem(dust_cx - 65.0, dust_y - 6.0, 20.0, Color::new(0.35, 0.85, 1.0, 1.0));
                draw_txt(&format!("STARDUST: {}", save_data.stardust), dust_cx - 45.0, dust_y, 22.0, Color::new(0.85, 0.95, 1.0, 1.0), f);

                // Desktop Keyboard Hint
                draw_centered("Press [SPACE] or Click to Play", cx, VIRTUAL_HEIGHT - 25.0, 16.0, Color::new(0.55, 0.55, 0.70, 0.65), f);
            }

            // ── Galaxy Map (Chapter Tabs) ────────────────────────────────────
            GameState::GalaxyMap => {
                let cx = VIRTUAL_WIDTH * 0.5;

                // Header
                draw_centered("GALAXY MAP", cx, 65.0, 36.0, WHITE, f);
                draw_centered("STELLAR CONSERVATORY", cx, 95.0, 18.0, Color::new(0.40, 0.85, 1.0, 0.85), f);

                // Back Button (Vector Arrow)
                let b_hov = hit(ptr, 20.0, 20.0, 90.0, 50.0);
                draw_rectangle(20.0, 20.0, 90.0, 50.0, if b_hov { Color::new(0.25, 0.25, 0.40, 0.90) } else { Color::new(0.12, 0.12, 0.24, 0.80) });
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
                    draw_rectangle_lines(tx, tab_y, tab_w, tab_h, if is_active { 2.0 } else { 1.0 }, if is_active { WHITE } else { Color::new(0.4, 0.4, 0.6, 0.5) });
                    draw_centered(chapter_name, tx + tab_w * 0.5, tab_y + 31.0, 18.0, if is_active { WHITE } else { Color::new(0.75, 0.75, 0.88, 0.8) }, f);
                }

                // 5 Sector Cards in Selected Chapter
                let start_idx = selected_chapter * 5;
                for row in 0..5 {
                    let idx = start_idx + row;
                    if idx >= sectors.len() { break; }
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
                    draw_rectangle_lines(card_x, card_y, card_w, card_h, if chov { 2.0 } else { 1.5 },
                        if chov { Color::new(0.55, 0.80, 1.0, 0.9) } else if unlocked { Color::new(0.35, 0.30, 0.65, 0.6) } else { Color::new(0.20, 0.20, 0.30, 0.4) });

                    if unlocked {
                        // Title & Lore
                        let title = format!("{}. {}", idx + 1, sec.name);
                        draw_txt(&title, card_x + 20.0, card_y + 34.0, 22.0, WHITE, f);
                        draw_txt(sec.lore, card_x + 20.0, card_y + 58.0, 15.0, Color::new(0.65, 0.70, 0.85, 0.75), f);

                        // Goal label
                        let goal_txt = sec.objective.display_text();
                        draw_txt(&format!("Goal: {}", goal_txt), card_x + 20.0, card_y + 84.0, 16.0, Color::new(0.40, 0.90, 0.60, 0.95), f);

                        // 3 Vector Stars on right
                        let star_start_x = card_x + card_w - 110.0;
                        for s in 0..3 {
                            let star_col = if (s as u8) < stars {
                                Color::new(1.0, 0.88, 0.22, 1.0)
                            } else {
                                Color::new(0.28, 0.28, 0.38, 0.65)
                            };
                            draw_vector_star(star_start_x + s as f32 * 36.0, card_y + 40.0, 14.0, star_col);
                        }
                    } else {
                        // Locked Sector Representation
                        draw_vector_lock(card_x + 60.0, card_y + 50.0, 32.0, Color::new(0.50, 0.50, 0.65, 0.60));
                        let title = format!("{}. {}", idx + 1, sec.name);
                        draw_txt(&title, card_x + 105.0, card_y + 45.0, 22.0, Color::new(0.55, 0.55, 0.65, 0.65), f);
                        draw_txt("Complete previous sector to unlock", card_x + 105.0, card_y + 72.0, 16.0, Color::new(0.45, 0.45, 0.55, 0.55), f);
                    }
                }

                // Bottom Stars & Stardust Summary
                let mut total_stars = 0u32;
                for i in 0..sectors.len() { total_stars += save_data.stars_for(i) as u32; }
                let stars_info = format!("STARS COLLECTED: {} / {}", total_stars, sectors.len() * 3);
                draw_centered(&stars_info, cx, VIRTUAL_HEIGHT - 60.0, 20.0, Color::new(1.0, 0.85, 0.30, 0.95), f);
            }

            // ── Playing ──────────────────────────────────────────────────────
            GameState::Playing => {
                let sec = &sectors[current_sector];

                // 1. Draw Settled Celestial Bodies
                for body in &bodies {
                    BodyRenderer::draw_body(body);
                }
                particles.draw();

                // 2. Container ("The Jar")
                let wall_w = 8.0;
                // Left Wall
                draw_rectangle(JAR_LEFT - wall_w, JAR_TOP_LINE, wall_w, JAR_HEIGHT, Color::new(0.28, 0.22, 0.55, 0.90));
                // Right Wall
                draw_rectangle(JAR_RIGHT, JAR_TOP_LINE, wall_w, JAR_HEIGHT, Color::new(0.28, 0.22, 0.55, 0.90));
                // Floor
                draw_rectangle(JAR_LEFT - wall_w, JAR_BOTTOM, JAR_WIDTH + wall_w * 2.0, 10.0, Color::new(0.28, 0.22, 0.55, 0.90));
                // Glass Inner Backdrop
                draw_rectangle(JAR_LEFT, JAR_TOP_LINE, JAR_WIDTH, JAR_HEIGHT, Color::new(0.04, 0.03, 0.10, 0.35));

                // Danger Line at JAR_TOP_LINE
                let pulse = if danger_timer > 0.0 { (get_time() as f32 * 12.0).sin().abs() * 0.6 + 0.4 } else { 0.35 };
                let dl_color = if danger_timer > 0.0 {
                    Color::new(1.0, 0.25, 0.25, pulse)
                } else {
                    Color::new(0.80, 0.60, 0.25, 0.40)
                };
                draw_line(JAR_LEFT, JAR_TOP_LINE, JAR_RIGHT, JAR_TOP_LINE, 2.5, dl_color);

                if danger_timer > 0.0 {
                    let secs_left = (CRITICAL_TIME_LIMIT - danger_timer).max(0.0);
                    let warn = format!("CONTAINMENT BREACH IN {:.1}s", secs_left);
                    draw_centered(&warn, VIRTUAL_WIDTH * 0.5, JAR_TOP_LINE - 14.0, 22.0, Color::new(1.0, 0.3, 0.3, 1.0), f);
                }

                // 3. Spawner & Comet Reserve Bar (y: 195..280)
                let ch_y = 195.0;
                let ch_w = 180.0;
                let ch_h = 80.0;

                // Choice A Card
                let a_x = JAR_LEFT;
                let a_sel = selected_choice == 0;
                let a_hov = hit(ptr, a_x, ch_y, ch_w, ch_h);
                let a_bg = if a_sel { Color::new(0.25, 0.80, 0.45, 0.90) } else if a_hov { Color::new(0.18, 0.22, 0.35, 0.85) } else { Color::new(0.12, 0.12, 0.24, 0.75) };
                draw_rectangle(a_x, ch_y, ch_w, ch_h, a_bg);
                draw_rectangle_lines(a_x, ch_y, ch_w, ch_h, if a_sel { 2.5 } else { 1.5 }, if a_sel { WHITE } else { Color::new(0.45, 0.45, 0.65, 0.5) });
                let ca = CelestialBody::new(101, choice_a, Vec2::new(a_x + 45.0, ch_y + 40.0), Vec2::ZERO);
                BodyRenderer::draw_body(&ca);
                draw_txt(choice_a.name(), a_x + 85.0, ch_y + 36.0, 18.0, WHITE, f);
                if a_sel { draw_txt("ACTIVE", a_x + 85.0, ch_y + 60.0, 14.0, Color::new(1.0, 0.95, 0.40, 1.0), f); }

                // Comet Reserve Slot (Center Circle)
                let res_cx = VIRTUAL_WIDTH * 0.5;
                let res_cy = ch_y + 40.0;
                let res_r = 38.0;
                draw_circle(res_cx, res_cy, res_r, Color::new(0.10, 0.08, 0.24, 0.90));
                draw_circle_lines(res_cx, res_cy, res_r, 2.0, Color::new(0.85, 0.65, 0.20, 0.85));

                if let Some(rt) = reserve {
                    let rb = CelestialBody::new(103, rt, Vec2::new(res_cx, res_cy), Vec2::ZERO);
                    BodyRenderer::draw_body(&rb);
                } else {
                    draw_centered("RESERVE", res_cx, res_cy - 6.0, 14.0, Color::new(0.70, 0.70, 0.85, 0.80), f);
                    draw_centered("BANK", res_cx, res_cy + 14.0, 16.0, Color::new(1.0, 0.85, 0.30, 0.90), f);
                }

                // Choice B Card
                let b_x = JAR_RIGHT - ch_w;
                let b_sel = selected_choice == 1;
                let b_hov = hit(ptr, b_x, ch_y, ch_w, ch_h);
                let b_bg = if b_sel { Color::new(0.25, 0.80, 0.45, 0.90) } else if b_hov { Color::new(0.18, 0.22, 0.35, 0.85) } else { Color::new(0.12, 0.12, 0.24, 0.75) };
                draw_rectangle(b_x, ch_y, ch_w, ch_h, b_bg);
                draw_rectangle_lines(b_x, ch_y, ch_w, ch_h, if b_sel { 2.5 } else { 1.5 }, if b_sel { WHITE } else { Color::new(0.45, 0.45, 0.65, 0.5) });
                let cb = CelestialBody::new(102, choice_b, Vec2::new(b_x + 45.0, ch_y + 40.0), Vec2::ZERO);
                BodyRenderer::draw_body(&cb);
                draw_txt(choice_b.name(), b_x + 85.0, ch_y + 36.0, 18.0, WHITE, f);
                if b_sel { draw_txt("ACTIVE", b_x + 85.0, ch_y + 60.0, 14.0, Color::new(1.0, 0.95, 0.40, 1.0), f); }

                // 4. Aim Guideline & Body Preview
                let active_tier = if selected_choice == 0 { choice_a } else { choice_b };
                let r_drop = active_tier.radius();

                // Trajectory Laser Line
                let guide_alpha = if is_aiming { 0.45 } else { 0.18 };
                draw_line(drop_x, DROP_Y + r_drop, drop_x, JAR_BOTTOM - r_drop, 2.0, Color::new(1.0, 1.0, 1.0, guide_alpha));

                // Aiming Preview Body
                let preview = CelestialBody::new(0, active_tier, Vec2::new(drop_x, DROP_Y), Vec2::ZERO);
                BodyRenderer::draw_body(&preview);

                // 5. HUD Top Bar & Objective Banner
                Hud::draw(current_score, save_data.high_score, save_data.stardust, sec.name, current_sector, &objective, danger_timer, f);

                // 6. Strategic Abilities Bar (y: 1140..1220, safely below jar bottom)
                let ab_y = 1140.0;
                let ab_w = 250.0;
                let ab_h = 70.0;

                // Wave Button
                let wave_avail = ability_charges > 0;
                let wave_hov = wave_avail && hit(ptr, JAR_LEFT, ab_y, ab_w, ab_h);
                let wave_col = if wave_hov {
                    Color::new(0.38, 0.75, 1.0, 1.0)
                } else if wave_avail {
                    Color::new(0.25, 0.58, 0.90, 0.90)
                } else {
                    Color::new(0.14, 0.14, 0.22, 0.55)
                };
                draw_rectangle(JAR_LEFT, ab_y, ab_w, ab_h, wave_col);
                draw_rectangle_lines(JAR_LEFT, ab_y, ab_w, ab_h, 1.8, if wave_avail { WHITE } else { Color::new(0.35, 0.35, 0.45, 0.5) });
                draw_centered("GRAVITY WAVE [1]", JAR_LEFT + ab_w * 0.5, ab_y + 28.0, 18.0, WHITE, f);
                draw_centered("Stabilize & lift stacks", JAR_LEFT + ab_w * 0.5, ab_y + 52.0, 14.0, Color::new(0.85, 0.92, 1.0, 0.85), f);

                // Flare Button
                let flare_x = JAR_RIGHT - ab_w;
                let flare_avail = ability_charges > 0;
                let flare_hov = flare_avail && hit(ptr, flare_x, ab_y, ab_w, ab_h);
                let flare_col = if flare_hov {
                    Color::new(1.0, 0.55, 0.25, 1.0)
                } else if flare_avail {
                    Color::new(0.92, 0.45, 0.18, 0.90)
                } else {
                    Color::new(0.14, 0.14, 0.22, 0.55)
                };
                draw_rectangle(flare_x, ab_y, ab_w, ab_h, flare_col);
                draw_rectangle_lines(flare_x, ab_y, ab_w, ab_h, 1.8, if flare_avail { WHITE } else { Color::new(0.35, 0.35, 0.45, 0.5) });
                draw_centered("SOLAR FLARE [2]", flare_x + ab_w * 0.5, ab_y + 28.0, 18.0, WHITE, f);
                draw_centered("Vaporise top 2 clutter", flare_x + ab_w * 0.5, ab_y + 52.0, 14.0, Color::new(0.85, 0.92, 1.0, 0.85), f);

                // Charges Indicator
                let charge_y = ab_y + ab_h + 26.0;
                let charge_txt = format!("ABILITY CHARGES: {} / {}", ability_charges, MAX_ABILITY_CHARGES);
                let charge_col = if ability_flash > 0.0 { Color::new(1.0, 0.88, 0.25, 1.0) } else { Color::new(0.70, 0.70, 0.85, 0.80) };
                draw_centered(&charge_txt, VIRTUAL_WIDTH * 0.5, charge_y, 18.0, charge_col, f);

                if ability_flash > 0.0 {
                    draw_centered("+ CHARGE EARNED!", VIRTUAL_WIDTH * 0.5, ab_y - 12.0, 22.0, Color::new(0.35, 1.0, 0.55, ability_flash / 0.9), f);
                }
            }

            // ── Sector Complete ──────────────────────────────────────────────
            GameState::SectorComplete => {
                for body in &bodies { BodyRenderer::draw_body(body); }
                particles.draw();

                draw_rectangle(0.0, 0.0, VIRTUAL_WIDTH, VIRTUAL_HEIGHT, Color::new(0.0, 0.0, 0.0, 0.82));
                let cx = VIRTUAL_WIDTH * 0.5;
                let sec = &sectors[current_sector];

                let pulse = ((sector_complete_timer * 4.0).sin() * 0.08 + 0.96).max(0.0);
                draw_centered("SECTOR RESTORED!", cx, VIRTUAL_HEIGHT * 0.26, 44.0 * pulse, Color::new(0.35, 1.0, 0.55, 1.0), f);
                draw_centered(sec.name, cx, VIRTUAL_HEIGHT * 0.26 + 60.0, 32.0, WHITE, f);
                draw_centered(sec.region, cx, VIRTUAL_HEIGHT * 0.26 + 92.0, 20.0, Color::new(0.40, 0.85, 1.0, 0.85), f);

                // 3 Celebratory Vector Stars
                let star_start_x = cx - 72.0;
                for s in 0..3 {
                    let col = if (s as u8) < earned_stars { Color::new(1.0, 0.88, 0.22, 1.0) } else { Color::new(0.28, 0.28, 0.38, 0.60) };
                    draw_vector_star(star_start_x + s as f32 * 72.0, VIRTUAL_HEIGHT * 0.26 + 155.0, 26.0, col);
                }

                let score_str = format!("Final Score: {}", current_score);
                draw_centered(&score_str, cx, VIRTUAL_HEIGHT * 0.44, 28.0, Color::new(1.0, 0.85, 0.30, 1.0), f);
                let dust_str = format!("+{} Stardust Earned", run_stardust);
                draw_centered(&dust_str, cx, VIRTUAL_HEIGHT * 0.44 + 40.0, 24.0, Color::new(0.40, 0.90, 1.0, 1.0), f);

                // Next Sector button
                let nx_y = VIRTUAL_HEIGHT * 0.65;
                let btn_avail = current_sector + 1 < sectors.len();
                let nx_label = if btn_avail { "NEXT SECTOR" } else { "GALAXY COMPLETED" };
                let nxh = hit(ptr, cx - 150.0, nx_y, 300.0, 66.0);
                draw_rectangle(cx - 150.0, nx_y, 300.0, 66.0, if nxh { Color::new(0.28, 0.85, 0.45, 1.0) } else { Color::new(0.18, 0.68, 0.34, 1.0) });
                draw_rectangle_lines(cx - 150.0, nx_y, 300.0, 66.0, 2.0, WHITE);
                draw_centered(nx_label, cx, nx_y + 42.0, 26.0, WHITE, f);

                // Galaxy Map button
                let mp_y = nx_y + 84.0;
                let mph = hit(ptr, cx - 150.0, mp_y, 300.0, 60.0);
                draw_rectangle(cx - 150.0, mp_y, 300.0, 60.0, if mph { Color::new(0.42, 0.22, 0.72, 1.0) } else { Color::new(0.30, 0.15, 0.55, 1.0) });
                draw_rectangle_lines(cx - 150.0, mp_y, 300.0, 60.0, 1.8, WHITE);
                draw_centered("GALAXY MAP", cx, mp_y + 38.0, 22.0, WHITE, f);
            }

            // ── Paused ───────────────────────────────────────────────────────
            GameState::Paused => {
                for body in &bodies { BodyRenderer::draw_body(body); }
                particles.draw();
                draw_rectangle(0.0, 0.0, VIRTUAL_WIDTH, VIRTUAL_HEIGHT, Color::new(0.0, 0.0, 0.0, 0.75));
                let cx = VIRTUAL_WIDTH * 0.5;
                draw_centered("PAUSED", cx, VIRTUAL_HEIGHT * 0.32, 56.0, WHITE, f);

                let ry = VIRTUAL_HEIGHT * 0.42;
                let rh = hit(ptr, cx - 140.0, ry, 280.0, 64.0);
                draw_rectangle(cx - 140.0, ry, 280.0, 64.0, if rh { Color::new(0.28, 0.80, 0.45, 1.0) } else { Color::new(0.18, 0.62, 0.34, 1.0) });
                draw_rectangle_lines(cx - 140.0, ry, 280.0, 64.0, 2.0, WHITE);
                draw_centered("RESUME", cx, ry + 42.0, 28.0, WHITE, f);

                let qy = ry + 84.0;
                let qh = hit(ptr, cx - 140.0, qy, 280.0, 64.0);
                draw_rectangle(cx - 140.0, qy, 280.0, 64.0, if qh { Color::new(0.72, 0.22, 0.22, 1.0) } else { Color::new(0.55, 0.14, 0.14, 1.0) });
                draw_rectangle_lines(cx - 140.0, qy, 280.0, 64.0, 1.8, WHITE);
                draw_centered("GALAXY MAP", cx, qy + 42.0, 26.0, WHITE, f);
            }

            // ── GameOver ─────────────────────────────────────────────────────
            GameState::GameOver => {
                for body in &bodies { BodyRenderer::draw_body(body); }
                particles.draw();
                go_action = GameOverModal::draw(current_score, save_data.high_score, run_stardust,
                    revives_used < FREE_REVIVES_PER_RUN, stardust_doubled, ptr, tap, f);
            }

            // ── Shop ─────────────────────────────────────────────────────────
            GameState::Shop => {
                shop_action = ShopModal::draw(&catalog, save_data.stardust, billing.is_ad_removed(),
                    &save_data.equipped_skin, &save_data.unlocked_skins,
                    shop_msg.as_deref(), ptr, tap, f);
            }

            // ── Ad Overlay ───────────────────────────────────────────────────
            GameState::WatchingAd => {
                let reward_label = if revives_used == 0 { "Sector Revival" } else { "2x Stardust" };
                ad_result = MockAdOverlay::draw(ads.get_ad_time_remaining(), 5.0, reward_label, ptr, tap, f);
            }
        }

        // ── Resolve Modal Actions ────────────────────────────────────────────
        match go_action {
            GameOverAction::WatchAdRevive => {
                ads.start_rewarded_ad(RewardType::EventHorizonRevive);
                game_state = GameState::WatchingAd;
            }
            GameOverAction::WatchAdDoubleStardust => {
                ads.start_rewarded_ad(RewardType::DoubleStardust);
                game_state = GameState::WatchingAd;
            }
            GameOverAction::Restart => {
                let sec = &sectors[current_sector];
                objective = ObjectiveTracker::new(sec.objective.clone());
                bodies.clear(); next_body_id = 1; current_score = 0; run_stardust = 0;
                revives_used = 0; stardust_doubled = false; danger_timer = 0.0;
                ability_charges = 0; drop_cooldown = 0.0;
                choice_a = CelestialTier::random_spawn_tier(); choice_b = CelestialTier::random_spawn_tier();
                selected_choice = 0; reserve = None; is_aiming = false;
                game_state = GameState::Playing;
            }
            GameOverAction::OpenShop => { game_state = GameState::Shop; }
            GameOverAction::None => {}
        }

        match shop_action {
            ShopAction::BuyItem(ref sku) => {
                if sku.contains("removeads") { billing.purchase_product(sku).ok(); save_data.ads_removed = true; }
                else if sku.contains("starpass") { billing.purchase_product(sku).ok(); save_data.celestial_pass_unlocked = true; }
                else if sku.ends_with(".500") { save_data.stardust += 500; }
                else if sku.ends_with(".2000") { save_data.stardust += 2000; }
                else if let Some(item) = catalog.skin_items.iter().find(|i| i.id == *sku) {
                    if save_data.stardust >= item.stardust_price && !save_data.unlocked_skins.contains(sku) {
                        save_data.stardust -= item.stardust_price;
                        save_data.unlocked_skins.push(sku.clone());
                        save_data.equipped_skin = sku.clone();
                        shop_msg = Some(format!("Equipped {}!", item.title));
                    }
                }
                let _ = save_mgr.save(&save_data);
            }
            ShopAction::EquipSkin(ref sid) => {
                save_data.equipped_skin = sid.clone();
                let _ = save_mgr.save(&save_data);
            }
            ShopAction::Close => {
                shop_msg = None;
                game_state = if current_score > 0 { GameState::GameOver } else { GameState::MainMenu };
            }
            ShopAction::None => {}
        }

        match ad_result {
            MockAdResult::ClaimReward => {
                if let Some(rw) = ads.skip_or_finish() {
                    handle_reward(rw, &mut current_score, &mut run_stardust, &mut revives_used,
                        &mut stardust_doubled, &mut bodies, &mut danger_timer,
                        &mut save_data, &save_mgr, &mut game_state);
                }
            }
            MockAdResult::CancelEarly => { ads.skip_or_finish(); game_state = GameState::GameOver; }
            MockAdResult::None => {}
        }

        // ── Blit Virtual Canvas to Physical Screen ───────────────────────────
        set_default_camera();
        clear_background(Color::new(0.02, 0.02, 0.06, 1.0));
        draw_texture_ex(&vt.texture, ox, oy, WHITE, DrawTextureParams {
            dest_size: Some(vec2(VIRTUAL_WIDTH * scale, VIRTUAL_HEIGHT * scale)),
            flip_y: true,
            ..Default::default()
        });

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
        RewardType::DebugAd => { *game_state = GameState::GameOver; }
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
