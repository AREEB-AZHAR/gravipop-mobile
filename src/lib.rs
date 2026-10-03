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
use crate::physics::*;
use crate::graphics::*;
use crate::audio::AudioEngine;
use crate::monetization::ads::{AdService, MockAdService, RewardType};
use crate::monetization::billing::{BillingService, MockBillingService};
use crate::monetization::economy::EconomyCatalog;
use crate::ui::*;

// ─────────────────────────────────────────────────────────────────────────────
//  Window Configuration
// ─────────────────────────────────────────────────────────────────────────────

pub fn window_conf() -> Conf {
    Conf {
        window_title: "GraviPop: Celestial Merge".to_string(),
        window_width: 450,
        window_height: 800,
        window_resizable: true,
        high_dpi: true,
        ..Default::default()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Drawing Utilities (shared across game and UI)
// ─────────────────────────────────────────────────────────────────────────────

/// Draw text centered horizontally around `cx`.
pub fn draw_centered(text: &str, cx: f32, y: f32, size: f32, color: Color, font: Option<&Font>) {
    let sz = size as u16;
    let dim = measure_text(text, font, sz, 1.0);
    draw_text_ex(text, cx - dim.width * 0.5, y, TextParams {
        font,
        font_size: sz,
        color,
        ..Default::default()
    });
}

/// Draw text at an absolute position using the active font.
pub fn draw_txt(text: &str, x: f32, y: f32, size: f32, color: Color, font: Option<&Font>) {
    draw_text_ex(text, x, y, TextParams {
        font,
        font_size: size as u16,
        color,
        ..Default::default()
    });
}

pub fn hit(pos: Vec2, x: f32, y: f32, w: f32, h: f32) -> bool {
    pos.x >= x && pos.x <= x + w && pos.y >= y && pos.y <= y + h
}

// ─────────────────────────────────────────────────────────────────────────────
//  Main Game Loop
// ─────────────────────────────────────────────────────────────────────────────

pub async fn game_main() {
    // ── Font Loading ──────────────────────────────────────────────────────────
    // On Android assets are accessed from the APK bundle directly.
    // On desktop we look in the assets/ sub-folder.
    let font_path = if cfg!(target_os = "android") {
        "font.ttf"
    } else {
        "assets/font.ttf"
    };
    let font: Option<Font> = load_ttf_font(font_path).await.ok();
    let f: Option<&Font> = font.as_ref();

    // ── Persistence & Systems ─────────────────────────────────────────────────
    let save_mgr = SaveManager::new();
    let mut save_data = save_mgr.load();
    let audio = AudioEngine::new().await;
    let mut starfield = Starfield::new();
    let mut particles = ParticleEngine::new();
    let gravity = GravityEngine::new(Vec2::new(VIRTUAL_WIDTH * 0.5, VIRTUAL_HEIGHT * 0.48));
    let mut ads = MockAdService::new();
    let mut billing = MockBillingService::new(
        save_data.ads_removed,
        save_data.celestial_pass_unlocked,
    );
    let catalog = EconomyCatalog::new();

    // ── Virtual Render Target (720 × 1280) ───────────────────────────────────
    let virtual_target = render_target(VIRTUAL_WIDTH as u32, VIRTUAL_HEIGHT as u32);
    virtual_target.texture.set_filter(FilterMode::Linear);
    let virtual_camera = Camera2D {
        render_target: Some(virtual_target.clone()),
        zoom: vec2(2.0 / VIRTUAL_WIDTH, -2.0 / VIRTUAL_HEIGHT),
        target: vec2(VIRTUAL_WIDTH * 0.5, VIRTUAL_HEIGHT * 0.5),
        ..Default::default()
    };

    // ── Session State ─────────────────────────────────────────────────────────
    let mut game_state = GameState::MainMenu;
    let mut bodies: Vec<CelestialBody> = Vec::new();
    let mut next_body_id = 1u64;
    let mut current_score = 0u64;
    let mut run_stardust = 0u64;
    let mut combo_count = 0usize;
    let mut combo_timer = 0.0f32;
    let mut critical_timer = 0.0f32;
    let mut revives_used = 0u32;
    let mut stardust_doubled = false;
    let mut shop_status_msg: Option<String> = None;

    // Slingshot
    let launch_pos = Vec2::new(VIRTUAL_WIDTH * 0.5, VIRTUAL_HEIGHT - 140.0);
    let mut queued_tier = CelestialTier::random_spawn_tier();
    let mut next_preview_tier = CelestialTier::random_spawn_tier();
    let mut is_dragging = false;
    let mut sling_drag = launch_pos;

    // ── Game Loop ─────────────────────────────────────────────────────────────
    loop {
        let dt = get_frame_time().min(0.05);

        // 1. Screen → Virtual coordinate mapping
        let sw = screen_width();
        let sh = screen_height();
        let scale = (sw / VIRTUAL_WIDTH).min(sh / VIRTUAL_HEIGHT);
        let ox = (sw - VIRTUAL_WIDTH * scale) * 0.5;
        let oy = (sh - VIRTUAL_HEIGHT * scale) * 0.5;

        let raw_m = Vec2::new(mouse_position().0, mouse_position().1);
        let vm = Vec2::new((raw_m.x - ox) / scale, (raw_m.y - oy) / scale);
        let clicked = is_mouse_button_pressed(MouseButton::Left);
        let held = is_mouse_button_down(MouseButton::Left);
        let released = is_mouse_button_released(MouseButton::Left);

        let touch_list = touches();
        let has_touch = !touch_list.is_empty();
        let touch_vp = if has_touch {
            let t = &touch_list[0];
            Some(Vec2::new((t.position.x - ox) / scale, (t.position.y - oy) / scale))
        } else {
            None
        };
        let ptr = touch_vp.unwrap_or(vm); // unified pointer (touch or mouse)

        // 2. World updates
        starfield.update(dt);
        particles.update(dt);

        // Ad tick — may immediately grant a reward
        if let Some(reward) = ads.update(dt) {
            handle_reward(
                reward, &mut current_score, &mut run_stardust, &mut revives_used,
                &mut stardust_doubled, &mut bodies, &mut critical_timer,
                &mut save_data, &save_mgr, &mut game_state,
            );
        }

        // 3. ── LOGIC state machine ─────────────────────────────────────────
        match game_state {
            GameState::MainMenu => {
                if clicked {
                    let bw = 300.0;
                    let bx = VIRTUAL_WIDTH * 0.5 - bw * 0.5;
                    let play_y = VIRTUAL_HEIGHT * 0.62;
                    if hit(ptr, bx, play_y, bw, 72.0) {
                        bodies.clear();
                        current_score = 0;
                        run_stardust = 0;
                        combo_count = 0;
                        combo_timer = 0.0;
                        critical_timer = 0.0;
                        revives_used = 0;
                        stardust_doubled = false;
                        game_state = GameState::Playing;
                    }
                    let shop_y = play_y + 92.0;
                    if hit(ptr, bx, shop_y, bw, 64.0) {
                        game_state = GameState::Shop;
                    }
                }
            }

            GameState::Playing => {
                // Combo decay
                if combo_timer > 0.0 {
                    combo_timer -= dt;
                    if combo_timer <= 0.0 { combo_count = 0; }
                }

                // Physics
                gravity.apply_forces(&mut bodies, dt);
                for b in bodies.iter_mut() { b.update(dt); }

                // Merge resolution
                let fusions = CollisionEngine::resolve_collisions(&mut bodies);
                for f in fusions {
                    combo_count += 1;
                    combo_timer = COMBO_TIMEOUT_SECS;
                    let mul = combo_count.min(8) as u64;
                    let pts = f.score_awarded * mul;
                    current_score += pts;
                    run_stardust += f.stardust_awarded as u64;
                    particles.spawn_fusion_burst(f.pos, f.new_tier.primary_color(), f.new_tier as usize);
                    let lbl = if mul > 1 {
                        format!("+{} x{} COMBO", pts, mul)
                    } else {
                        format!("+{}", pts)
                    };
                    particles.add_floating_text(lbl, f.pos, f.new_tier.glow_color(), 28.0);
                    audio.play_fusion_chime(combo_count);
                }

                // Event Horizon overflow check
                let outside = bodies.iter().any(|b| b.outside_timer > 0.5);
                if outside {
                    critical_timer += dt;
                    if critical_timer >= CRITICAL_TIME_LIMIT {
                        audio.play_game_over();
                        save_data.runs_played += 1;
                        if current_score > save_data.high_score {
                            save_data.high_score = current_score;
                        }
                        save_data.stardust += run_stardust;
                        let _ = save_mgr.save(&save_data);
                        if !billing.is_ad_removed()
                            && save_data.runs_played % INTERSTITIAL_RUN_INTERVAL == 0
                        {
                            ads.start_interstitial_ad();
                            game_state = GameState::WatchingAd;
                        } else {
                            game_state = GameState::GameOver;
                        }
                    }
                } else {
                    critical_timer = (critical_timer - dt * 1.5).max(0.0);
                }

                // Slingshot input
                if held || has_touch {
                    if !is_dragging {
                        if (ptr - launch_pos).length() < 90.0 {
                            is_dragging = true;
                            sling_drag = ptr;
                        }
                    } else {
                        let pull = ptr - launch_pos;
                        let clamped = pull.length().min(160.0);
                        sling_drag = launch_pos + pull.normalize_or_zero() * clamped;
                    }
                }
                if (released || (!has_touch && is_dragging)) && is_dragging {
                    is_dragging = false;
                    let pull = launch_pos - sling_drag;
                    if pull.length() > 20.0 {
                        let speed = (pull.length() * SLING_SENSITIVITY).min(MAX_SLING_SPEED);
                        let vel = pull.normalize() * speed;
                        bodies.push(CelestialBody::new(next_body_id, queued_tier, launch_pos, vel));
                        next_body_id += 1;
                        audio.play_slingshot();
                        particles.spawn_trail(launch_pos, queued_tier.primary_color());
                        queued_tier = next_preview_tier;
                        next_preview_tier = CelestialTier::random_spawn_tier();
                    }
                    sling_drag = launch_pos;
                }

                // Pause button (top-right)
                if clicked && hit(ptr, VIRTUAL_WIDTH - 72.0, 20.0, 52.0, 52.0) {
                    game_state = GameState::Paused;
                }
            }

            GameState::Paused => {
                if clicked {
                    let bx = VIRTUAL_WIDTH * 0.5 - 130.0;
                    let ry = VIRTUAL_HEIGHT * 0.42;
                    if hit(ptr, bx, ry, 260.0, 66.0) {
                        game_state = GameState::Playing;
                    }
                    let qy = ry + 86.0;
                    if hit(ptr, bx, qy, 260.0, 66.0) {
                        game_state = GameState::MainMenu;
                    }
                }
            }

            // GameOver / Shop / WatchingAd logic handled in render section
            _ => {}
        }

        // 4. ── RENDER to virtual buffer ────────────────────────────────────
        set_camera(&virtual_camera);
        let danger = (critical_timer / CRITICAL_TIME_LIMIT).clamp(0.0, 1.0);
        starfield.draw(gravity.center, danger);

        for body in &bodies {
            BodyRenderer::draw_body(body);
        }
        particles.draw();

        // Accumulated immediate-mode actions from modal UIs
        let mut go_action = GameOverAction::None;
        let mut shop_action = ShopAction::None;
        let mut ad_result = MockAdResult::None;

        match game_state {
            // ── In-Game ───────────────────────────────────────────────────────
            GameState::Playing => {
                // Launch bay indicator
                draw_circle_lines(
                    launch_pos.x, launch_pos.y, 40.0, 2.0,
                    Color::new(0.4, 0.65, 1.0, 0.4),
                );

                if is_dragging {
                    let pull = launch_pos - sling_drag;
                    let spd = (pull.length() * SLING_SENSITIVITY).min(MAX_SLING_SPEED);
                    let iv = pull.normalize_or_zero() * spd;
                    let traj = gravity.predict_trajectory(launch_pos, iv, 28, 0.035);
                    BodyRenderer::draw_slingshot_trajectory(launch_pos, sling_drag, &traj, queued_tier);
                } else {
                    BodyRenderer::draw_body(&CelestialBody::new(0, queued_tier, launch_pos, Vec2::ZERO));
                }

                Hud::draw(
                    current_score,
                    save_data.high_score,
                    save_data.stardust,
                    combo_count,
                    combo_timer,
                    critical_timer,
                    next_preview_tier,
                    f,
                );

                // Pause button
                let px = VIRTUAL_WIDTH - 72.0;
                let py = 20.0;
                draw_rectangle(px, py, 52.0, 52.0, Color::new(0.10, 0.10, 0.22, 0.80));
                draw_rectangle_lines(px, py, 52.0, 52.0, 1.5, Color::new(0.5, 0.5, 0.75, 0.7));
                draw_txt("||", px + 14.0, py + 37.0, 30.0, WHITE, f);
            }

            // ── Main Menu ─────────────────────────────────────────────────────
            GameState::MainMenu => {
                let cx = VIRTUAL_WIDTH * 0.5;
                let ty = VIRTUAL_HEIGHT * 0.30;

                // Title card background
                draw_rectangle(
                    40.0, ty - 80.0, VIRTUAL_WIDTH - 80.0, 200.0,
                    Color::new(0.05, 0.04, 0.14, 0.70),
                );
                draw_rectangle_lines(
                    40.0, ty - 80.0, VIRTUAL_WIDTH - 80.0, 200.0,
                    1.5, Color::new(0.45, 0.35, 0.75, 0.40),
                );

                draw_centered("GRAVIPOP", cx, ty, 80.0, WHITE, f);
                draw_centered(
                    "CELESTIAL MERGE", cx, ty + 55.0, 30.0,
                    Color::new(0.45, 0.88, 1.0, 0.95), f,
                );

                let hi = format!("HIGH SCORE    {}", save_data.high_score);
                draw_centered(&hi, cx, ty + 118.0, 26.0, Color::new(0.95, 0.82, 0.30, 1.0), f);

                let dust = format!("STARDUST    {}", save_data.stardust);
                draw_centered(&dust, cx, ty + 150.0, 24.0, Color::new(0.42, 0.90, 1.0, 1.0), f);

                // PLAY button
                let bw = 300.0;
                let bx = cx - bw * 0.5;
                let play_y = VIRTUAL_HEIGHT * 0.62;
                let ph = is_in_game_rect(ptr, bx, play_y, bw, 72.0);
                draw_rectangle(
                    bx, play_y, bw, 72.0,
                    if ph { Color::new(0.28, 0.90, 0.50, 1.0) } else { Color::new(0.18, 0.72, 0.36, 1.0) },
                );
                draw_rectangle_lines(bx, play_y, bw, 72.0, 2.5, Color::new(1.0, 1.0, 1.0, 0.85));
                draw_centered("PLAY NOW", cx, play_y + 47.0, 32.0, WHITE, f);

                // SHOP button
                let shop_y = play_y + 92.0;
                let sh2 = is_in_game_rect(ptr, bx, shop_y, bw, 64.0);
                draw_rectangle(
                    bx, shop_y, bw, 64.0,
                    if sh2 { Color::new(0.55, 0.28, 0.86, 1.0) } else { Color::new(0.40, 0.20, 0.68, 1.0) },
                );
                draw_rectangle_lines(bx, shop_y, bw, 64.0, 1.8, Color::new(1.0, 1.0, 1.0, 0.65));
                draw_centered("COSMIC SHOP", cx, shop_y + 41.0, 28.0, WHITE, f);

                // Footer note
                let (foot, fcol) = if billing.is_ad_removed() {
                    ("PREMIUM AD-FREE EDITION", Color::new(0.40, 1.0, 0.60, 0.90))
                } else {
                    ("Open Shop to Remove Ads", Color::new(0.65, 0.65, 0.80, 0.70))
                };
                draw_centered(foot, cx, VIRTUAL_HEIGHT - 50.0, 20.0, fcol, f);
            }

            // ── Paused ────────────────────────────────────────────────────────
            GameState::Paused => {
                draw_rectangle(0.0, 0.0, VIRTUAL_WIDTH, VIRTUAL_HEIGHT, Color::new(0.0, 0.0, 0.0, 0.72));
                let cx = VIRTUAL_WIDTH * 0.5;
                draw_centered("PAUSED", cx, VIRTUAL_HEIGHT * 0.34, 56.0, WHITE, f);

                let bx = cx - 130.0;
                let ry = VIRTUAL_HEIGHT * 0.44;
                let rh = is_in_game_rect(ptr, bx, ry, 260.0, 66.0);
                draw_rectangle(
                    bx, ry, 260.0, 66.0,
                    if rh { Color::new(0.28, 0.75, 0.44, 1.0) } else { Color::new(0.18, 0.60, 0.32, 1.0) },
                );
                draw_rectangle_lines(bx, ry, 260.0, 66.0, 1.8, WHITE);
                draw_centered("RESUME", cx, ry + 43.0, 28.0, WHITE, f);

                let qy = ry + 86.0;
                let qh = is_in_game_rect(ptr, bx, qy, 260.0, 66.0);
                draw_rectangle(
                    bx, qy, 260.0, 66.0,
                    if qh { Color::new(0.76, 0.26, 0.26, 1.0) } else { Color::new(0.60, 0.18, 0.18, 1.0) },
                );
                draw_rectangle_lines(bx, qy, 260.0, 66.0, 1.8, WHITE);
                draw_centered("MAIN MENU", cx, qy + 43.0, 28.0, WHITE, f);
            }

            // ── Game Over ─────────────────────────────────────────────────────
            GameState::GameOver => {
                go_action = GameOverModal::draw(
                    current_score,
                    save_data.high_score,
                    run_stardust,
                    revives_used < FREE_REVIVES_PER_RUN,
                    stardust_doubled,
                    ptr,
                    clicked,
                    f,
                );
            }

            // ── Shop ─────────────────────────────────────────────────────────
            GameState::Shop => {
                shop_action = ShopModal::draw(
                    &catalog,
                    save_data.stardust,
                    billing.is_ad_removed(),
                    &save_data.equipped_skin,
                    &save_data.unlocked_skins,
                    shop_status_msg.as_deref(),
                    ptr,
                    clicked,
                    f,
                );
            }

            // ── Watching Ad ───────────────────────────────────────────────────
            GameState::WatchingAd => {
                let reward_label = if revives_used == 0 && critical_timer >= CRITICAL_TIME_LIMIT {
                    "Singularity Rewind & Revive"
                } else if !stardust_doubled && run_stardust > 0 {
                    "2x Cosmic Stardust"
                } else {
                    "Cosmic Stardust Bounty"
                };
                ad_result = MockAdOverlay::draw(
                    ads.get_ad_time_remaining(),
                    5.0,
                    reward_label,
                    ptr,
                    clicked,
                    f,
                );
            }
        }

        // 5. ── Resolve modal actions ──────────────────────────────────────────
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
                bodies.clear();
                current_score = 0;
                run_stardust = 0;
                combo_count = 0;
                combo_timer = 0.0;
                critical_timer = 0.0;
                revives_used = 0;
                stardust_doubled = false;
                game_state = GameState::Playing;
            }
            GameOverAction::OpenShop => { game_state = GameState::Shop; }
            GameOverAction::None => {}
        }

        match shop_action {
            ShopAction::BuyItem(ref sku) => {
                if sku.contains("removeads") || sku.contains("starpass")
                    || sku.starts_with("com.gravipop.stardust")
                {
                    match billing.purchase_product(sku) {
                        Ok(msg) => {
                            if sku.contains("removeads") { save_data.ads_removed = true; }
                            if sku.contains("starpass") { save_data.celestial_pass_unlocked = true; }
                            if sku.ends_with(".500") { save_data.stardust += 500; }
                            else if sku.ends_with(".2000") { save_data.stardust += 2000; }
                            let _ = save_mgr.save(&save_data);
                            shop_status_msg = Some(msg);
                        }
                        Err(e) => { shop_status_msg = Some(format!("Purchase failed: {}", e)); }
                    }
                } else if let Some(item) = catalog.skin_items.iter().find(|i| i.id == *sku) {
                    if save_data.stardust >= item.stardust_price
                        && !save_data.unlocked_skins.contains(sku)
                    {
                        save_data.stardust -= item.stardust_price;
                        save_data.unlocked_skins.push(sku.clone());
                        save_data.equipped_skin = sku.clone();
                        let _ = save_mgr.save(&save_data);
                        shop_status_msg = Some(format!("Equipped {}!", item.title));
                    }
                }
            }
            ShopAction::EquipSkin(ref sid) => {
                save_data.equipped_skin = sid.clone();
                let _ = save_mgr.save(&save_data);
                shop_status_msg = Some(format!("Equipped {} theme!", sid));
            }
            ShopAction::Close => {
                shop_status_msg = None;
                game_state = if current_score > 0 { GameState::GameOver } else { GameState::MainMenu };
            }
            ShopAction::None => {}
        }

        match ad_result {
            MockAdResult::ClaimReward => {
                if let Some(reward) = ads.skip_or_finish() {
                    handle_reward(
                        reward, &mut current_score, &mut run_stardust, &mut revives_used,
                        &mut stardust_doubled, &mut bodies, &mut critical_timer,
                        &mut save_data, &save_mgr, &mut game_state,
                    );
                }
            }
            MockAdResult::CancelEarly => {
                ads.skip_or_finish();
                game_state = GameState::GameOver;
            }
            MockAdResult::None => {}
        }

        // 6. ── Blit virtual buffer → physical screen ──────────────────────────
        set_default_camera();
        clear_background(Color::new(0.02, 0.02, 0.06, 1.0));
        draw_texture_ex(
            &virtual_target.texture,
            ox, oy, WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(VIRTUAL_WIDTH * scale, VIRTUAL_HEIGHT * scale)),
                flip_y: true,
                ..Default::default()
            },
        );

        next_frame().await;
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Helpers
// ─────────────────────────────────────────────────────────────────────────────

fn is_in_game_rect(pos: Vec2, x: f32, y: f32, w: f32, h: f32) -> bool {
    pos.x >= x && pos.x <= x + w && pos.y >= y && pos.y <= y + h
}

pub fn handle_reward(
    reward: RewardType,
    _current_score: &mut u64,
    run_stardust: &mut u64,
    revives_used: &mut u32,
    stardust_doubled: &mut bool,
    bodies: &mut Vec<CelestialBody>,
    critical_timer: &mut f32,
    save_data: &mut SaveData,
    save_mgr: &SaveManager,
    game_state: &mut GameState,
) {
    match reward {
        RewardType::EventHorizonRevive => {
            *revives_used += 1;
            *critical_timer = 0.0;
            bodies.sort_by(|a, b| {
                b.pos.y.partial_cmp(&a.pos.y).unwrap_or(std::cmp::Ordering::Equal)
            });
            let n = (bodies.len() / 3).max(1);
            bodies.truncate(bodies.len().saturating_sub(n));
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
        RewardType::DebugAd => {
            *game_state = GameState::GameOver;
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Android Entry Point
//  On Android, NativeActivity calls android_main().  We MUST actually start
//  the macroquad event loop here, or the app will crash immediately.
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(target_os = "android")]
#[no_mangle]
pub extern "C" fn android_main(_app: *mut std::ffi::c_void) {
    macroquad::Window::from_config(window_conf(), game_main());
}

// ─────────────────────────────────────────────────────────────────────────────
//  Unit Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::*;
    use crate::core::game_state::GameState;
    use crate::core::save_system::SaveData;
    use crate::physics::celestial_tier::CelestialTier;

    #[test]
    fn test_hit_detection_inside() {
        let pos = Vec2::new(50.0, 50.0);
        assert!(hit(pos, 40.0, 40.0, 20.0, 20.0));
    }

    #[test]
    fn test_hit_detection_outside() {
        let pos = Vec2::new(10.0, 10.0);
        assert!(!hit(pos, 40.0, 40.0, 20.0, 20.0));
    }

    #[test]
    fn test_virtual_resolution_constants() {
        assert_eq!(VIRTUAL_WIDTH, 720.0);
        assert_eq!(VIRTUAL_HEIGHT, 1280.0);
    }

    #[test]
    fn test_game_state_transitions() {
        let state = GameState::MainMenu;
        assert_ne!(state, GameState::Playing);
    }

    #[test]
    fn test_stardust_accumulation() {
        let mut save = SaveData::default();
        save.stardust += 500;
        assert_eq!(save.stardust, 500);
    }

    #[test]
    fn test_celestial_tier_ordering() {
        let t1 = CelestialTier::Stardust;
        let t10 = CelestialTier::CosmicSingularity;
        assert!((t1 as usize) < (t10 as usize));
    }

    #[test]
    fn test_score_multiplier_cap() {
        let combo = 10usize;
        let capped = combo.min(8) as u64;
        assert_eq!(capped, 8);
    }

    #[test]
    fn test_critical_timer_bounds() {
        let timer: f32 = 3.5;
        let ratio = (timer / CRITICAL_TIME_LIMIT).clamp(0.0, 1.0);
        assert!(ratio <= 1.0);
    }

    #[test]
    fn test_sling_speed_cap() {
        let raw = 9999.0_f32;
        let capped = raw.min(MAX_SLING_SPEED);
        assert_eq!(capped, MAX_SLING_SPEED);
    }

    #[test]
    fn test_revive_guard() {
        let revives_used: u32 = 1;
        let available = revives_used < FREE_REVIVES_PER_RUN;
        assert!(!available);
    }

    #[test]
    fn test_high_score_update() {
        let mut save = SaveData::default();
        save.high_score = 1000;
        let new_score = 1500u64;
        if new_score > save.high_score { save.high_score = new_score; }
        assert_eq!(save.high_score, 1500);
    }
}
