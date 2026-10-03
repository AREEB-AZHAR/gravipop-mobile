mod core;
mod physics;
mod graphics;
mod audio;
mod monetization;
mod ui;

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

fn window_conf() -> Conf {
    Conf {
        window_title: "GraviPop: Celestial Merge".to_string(),
        window_width: 450,
        window_height: 800,
        window_resizable: true,
        high_dpi: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let save_mgr = SaveManager::new();
    let mut save_data = save_mgr.load();

    let audio = AudioEngine::new().await;
    let mut starfield = Starfield::new();
    let mut particles = ParticleEngine::new();
    let gravity = GravityEngine::new(Vec2::new(VIRTUAL_WIDTH * 0.5, VIRTUAL_HEIGHT * 0.48));

    let mut ads = MockAdService::new();
    let mut billing = MockBillingService::new(save_data.ads_removed, save_data.celestial_pass_unlocked);
    let catalog = EconomyCatalog::new();

    // Virtual High-Resolution Render Target
    let virtual_target = render_target(VIRTUAL_WIDTH as u32, VIRTUAL_HEIGHT as u32);
    virtual_target.texture.set_filter(FilterMode::Linear);

    let virtual_camera = Camera2D {
        render_target: Some(virtual_target.clone()),
        zoom: vec2(2.0 / VIRTUAL_WIDTH, -2.0 / VIRTUAL_HEIGHT),
        target: vec2(VIRTUAL_WIDTH * 0.5, VIRTUAL_HEIGHT * 0.5),
        ..Default::default()
    };

    // Game Session State
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

    // Slingshot launch bay state
    let launch_pos = Vec2::new(VIRTUAL_WIDTH * 0.5, VIRTUAL_HEIGHT - 130.0);
    let mut queued_tier = CelestialTier::random_spawn_tier();
    let mut next_preview_tier = CelestialTier::random_spawn_tier();
    let mut is_dragging_sling = false;
    let mut sling_drag_pos = launch_pos;

    loop {
        let dt = get_frame_time().min(0.05);

        // 1. Calculate Virtual Viewport & Pointer Coordinates
        let screen_w = screen_width();
        let screen_h = screen_height();
        let scale = (screen_w / VIRTUAL_WIDTH).min(screen_h / VIRTUAL_HEIGHT);
        let offset_x = (screen_w - VIRTUAL_WIDTH * scale) * 0.5;
        let offset_y = (screen_h - VIRTUAL_HEIGHT * scale) * 0.5;

        let raw_mouse = Vec2::new(mouse_position().0, mouse_position().1);
        let v_mouse = Vec2::new(
            (raw_mouse.x - offset_x) / scale,
            (raw_mouse.y - offset_y) / scale,
        );
        let mouse_clicked = is_mouse_button_pressed(MouseButton::Left);
        let mouse_down = is_mouse_button_down(MouseButton::Left);
        let mouse_released = is_mouse_button_released(MouseButton::Left);

        // Touch Input Support (Mobile Native Touch Screen)
        let has_touch = !touches().is_empty();
        let active_touch_pos = if has_touch {
            let t = &touches()[0];
            Some(Vec2::new((t.position.x - offset_x) / scale, (t.position.y - offset_y) / scale))
        } else {
            None
        };
        let effective_pointer = active_touch_pos.unwrap_or(v_mouse);

        // 2. Global Particle and Starfield Updates
        starfield.update(dt);
        particles.update(dt);

        // Background Ad Watch Check
        if let Some(reward) = ads.update(dt) {
            handle_reward(
                reward,
                &mut current_score,
                &mut run_stardust,
                &mut revives_used,
                &mut stardust_doubled,
                &mut bodies,
                &mut critical_timer,
                &mut save_data,
                &save_mgr,
                &mut game_state,
            );
        }

        // 3. State Machine Logic
        match game_state {
            GameState::MainMenu => {
                if mouse_clicked {
                    // Check PLAY Button
                    let play_btn_x = VIRTUAL_WIDTH * 0.5 - 130.0;
                    let play_btn_y = VIRTUAL_HEIGHT * 0.62;
                    if is_in_rect(effective_pointer, play_btn_x, play_btn_y, 260.0, 68.0) {
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

                    // Check STORE Button
                    let store_btn_y = play_btn_y + 85.0;
                    if is_in_rect(effective_pointer, play_btn_x, store_btn_y, 260.0, 56.0) {
                        game_state = GameState::Shop;
                    }
                }
            }

            GameState::Playing => {
                // Combo decay
                if combo_timer > 0.0 {
                    combo_timer -= dt;
                    if combo_timer <= 0.0 {
                        combo_count = 0;
                    }
                }

                // Physics Simulation
                gravity.apply_forces(&mut bodies, dt);

                for b in bodies.iter_mut() {
                    b.update(dt);
                }

                // Collisions and Fusions
                let fusions = CollisionEngine::resolve_collisions(&mut bodies);
                for f in fusions {
                    combo_count += 1;
                    combo_timer = COMBO_TIMEOUT_SECS;

                    let multiplier = combo_count.min(8) as u64;
                    let points = f.score_awarded * multiplier;
                    current_score += points;
                    run_stardust += f.stardust_awarded as u64;

                    particles.spawn_fusion_burst(f.pos, f.new_tier.primary_color(), f.new_tier as usize);
                    let label = if multiplier > 1 {
                        format!("+{} [x{} COMBO!]", points, multiplier)
                    } else {
                        format!("+{}", points)
                    };
                    particles.add_floating_text(label, f.pos, f.new_tier.glow_color(), 26.0);
                    audio.play_fusion_chime(combo_count);
                }

                // Event Horizon Overflow Check
                let has_outside = bodies.iter().any(|b| b.outside_timer > 0.5);
                if has_outside {
                    critical_timer += dt;
                    if critical_timer >= CRITICAL_TIME_LIMIT {
                        audio.play_game_over();
                        save_data.runs_played += 1;
                        if current_score > save_data.high_score {
                            save_data.high_score = current_score;
                        }
                        save_data.stardust += run_stardust;
                        let _ = save_mgr.save(&save_data);

                        if !billing.is_ad_removed() && save_data.runs_played % INTERSTITIAL_RUN_INTERVAL == 0 {
                            ads.start_interstitial_ad();
                            game_state = GameState::WatchingAd;
                        } else {
                            game_state = GameState::GameOver;
                        }
                    }
                } else {
                    critical_timer = (critical_timer - dt * 1.5).max(0.0);
                }

                // Slingshot Input Handling
                if mouse_down || has_touch {
                    let sling_radius = 85.0;
                    if !is_dragging_sling {
                        if (effective_pointer - launch_pos).length() < sling_radius {
                            is_dragging_sling = true;
                            sling_drag_pos = effective_pointer;
                        }
                    } else {
                        let pull = effective_pointer - launch_pos;
                        let dist = pull.length().min(160.0);
                        sling_drag_pos = launch_pos + pull.normalize_or_zero() * dist;
                    }
                }

                if (mouse_released || (!has_touch && is_dragging_sling)) && is_dragging_sling {
                    is_dragging_sling = false;
                    let pull = launch_pos - sling_drag_pos;
                    if pull.length() > 20.0 {
                        let speed = (pull.length() * SLING_SENSITIVITY).min(MAX_SLING_SPEED);
                        let vel = pull.normalize() * speed;

                        let new_id = next_body_id;
                        next_body_id += 1;

                        let new_body = CelestialBody::new(new_id, queued_tier, launch_pos, vel);
                        bodies.push(new_body);

                        audio.play_slingshot();
                        particles.spawn_trail(launch_pos, queued_tier.primary_color());

                        queued_tier = next_preview_tier;
                        next_preview_tier = CelestialTier::random_spawn_tier();
                    }
                    sling_drag_pos = launch_pos;
                }

                // Pause Button
                let pause_x = VIRTUAL_WIDTH - 65.0;
                let pause_y = 20.0;
                if mouse_clicked && is_in_rect(effective_pointer, pause_x, pause_y, 45.0, 45.0) {
                    game_state = GameState::Paused;
                }
            }

            GameState::Paused => {
                if mouse_clicked {
                    let res_x = VIRTUAL_WIDTH * 0.5 - 110.0;
                    let res_y = VIRTUAL_HEIGHT * 0.45;
                    if is_in_rect(effective_pointer, res_x, res_y, 220.0, 56.0) {
                        game_state = GameState::Playing;
                    }
                    let quit_y = res_y + 75.0;
                    if is_in_rect(effective_pointer, res_x, quit_y, 220.0, 56.0) {
                        game_state = GameState::MainMenu;
                    }
                }
            }

            GameState::GameOver => {}
            GameState::Shop => {}
            GameState::WatchingAd => {}
        }

        // =====================================================================
        // 4. RENDER TO VIRTUAL BUFFER (720 x 1280 Crisp Virtual Coordinate Space)
        // =====================================================================
        set_camera(&virtual_camera);

        let danger_ratio = (critical_timer / CRITICAL_TIME_LIMIT).clamp(0.0, 1.0);
        starfield.draw(gravity.center, danger_ratio);

        for body in &bodies {
            BodyRenderer::draw_body(body);
        }

        particles.draw();

        if game_state == GameState::Playing {
            draw_circle_lines(launch_pos.x, launch_pos.y, 35.0, 2.0, Color::new(0.4, 0.6, 0.9, 0.4));

            if is_dragging_sling {
                let pull = launch_pos - sling_drag_pos;
                let speed = (pull.length() * SLING_SENSITIVITY).min(MAX_SLING_SPEED);
                let initial_vel = pull.normalize_or_zero() * speed;
                let trajectory = gravity.predict_trajectory(launch_pos, initial_vel, 28, 0.035);
                BodyRenderer::draw_slingshot_trajectory(launch_pos, sling_drag_pos, &trajectory, queued_tier);
            } else {
                let launch_body = CelestialBody::new(0, queued_tier, launch_pos, Vec2::ZERO);
                BodyRenderer::draw_body(&launch_body);
            }

            Hud::draw(
                current_score,
                save_data.high_score,
                save_data.stardust,
                combo_count,
                combo_timer,
                critical_timer,
                next_preview_tier,
            );
        }

        // State-Specific Overlays
        match game_state {
            GameState::Playing => {}
            GameState::MainMenu => {
                let title_y = VIRTUAL_HEIGHT * 0.28;
                draw_text("GRAVIPOP", VIRTUAL_WIDTH * 0.5 - 165.0, title_y, 62.0, WHITE);
                draw_text("CELESTIAL MERGE", VIRTUAL_WIDTH * 0.5 - 145.0, title_y + 40.0, 26.0, Color::new(0.4, 0.85, 1.0, 0.9));

                let best_txt = format!("HIGH SCORE: {}", save_data.high_score);
                draw_text(&best_txt, VIRTUAL_WIDTH * 0.5 - 105.0, title_y + 90.0, 22.0, Color::new(0.95, 0.8, 0.3, 1.0));

                let dust_txt = format!("✦ STARDUST: {}", save_data.stardust);
                draw_text(&dust_txt, VIRTUAL_WIDTH * 0.5 - 85.0, title_y + 120.0, 22.0, Color::new(0.4, 0.9, 1.0, 1.0));

                // Play Button
                let play_x = VIRTUAL_WIDTH * 0.5 - 130.0;
                let play_y = VIRTUAL_HEIGHT * 0.62;
                let play_hover = is_in_rect(effective_pointer, play_x, play_y, 260.0, 68.0);
                draw_rectangle(play_x, play_y, 260.0, 68.0, if play_hover { Color::new(0.25, 0.85, 0.45, 1.0) } else { Color::new(0.18, 0.70, 0.35, 1.0) });
                draw_rectangle_lines(play_x, play_y, 260.0, 68.0, 2.0, WHITE);
                draw_text("▶ PLAY NOW", play_x + 55.0, play_y + 44.0, 26.0, WHITE);

                // Shop Button
                let store_y = play_y + 85.0;
                let store_hover = is_in_rect(effective_pointer, play_x, store_y, 260.0, 56.0);
                draw_rectangle(play_x, store_y, 260.0, 56.0, if store_hover { Color::new(0.5, 0.3, 0.8, 1.0) } else { Color::new(0.38, 0.22, 0.65, 1.0) });
                draw_rectangle_lines(play_x, store_y, 260.0, 56.0, 1.5, WHITE);
                draw_text("✦ COSMIC SHOP", play_x + 48.0, store_y + 36.0, 22.0, WHITE);

                if billing.is_ad_removed() {
                    draw_text("✓ PREMIUM AD-FREE EDITION", VIRTUAL_WIDTH * 0.5 - 120.0, VIRTUAL_HEIGHT - 60.0, 18.0, Color::new(0.4, 1.0, 0.6, 0.9));
                } else {
                    draw_text("Tap Store to Remove Ads & Support Dev", VIRTUAL_WIDTH * 0.5 - 150.0, VIRTUAL_HEIGHT - 60.0, 16.0, Color::new(0.7, 0.7, 0.8, 0.6));
                }
            }

            GameState::Paused => {
                draw_rectangle(0.0, 0.0, VIRTUAL_WIDTH, VIRTUAL_HEIGHT, Color::new(0.0, 0.0, 0.0, 0.7));
                draw_text("GAME PAUSED", VIRTUAL_WIDTH * 0.5 - 120.0, VIRTUAL_HEIGHT * 0.38, 36.0, WHITE);

                let res_x = VIRTUAL_WIDTH * 0.5 - 110.0;
                let res_y = VIRTUAL_HEIGHT * 0.45;
                draw_rectangle(res_x, res_y, 220.0, 56.0, Color::new(0.2, 0.65, 0.4, 1.0));
                draw_text("RESUME", res_x + 65.0, res_y + 36.0, 22.0, WHITE);

                let quit_y = res_y + 75.0;
                draw_rectangle(res_x, quit_y, 220.0, 56.0, Color::new(0.7, 0.25, 0.25, 1.0));
                draw_text("MAIN MENU", res_x + 50.0, quit_y + 36.0, 22.0, WHITE);
            }

            GameState::GameOver => {
                let action = GameOverModal::draw(
                    current_score,
                    save_data.high_score,
                    run_stardust,
                    revives_used < FREE_REVIVES_PER_RUN,
                    stardust_doubled,
                    effective_pointer,
                    mouse_clicked,
                );

                match action {
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
                    GameOverAction::OpenShop => {
                        game_state = GameState::Shop;
                    }
                    GameOverAction::None => {}
                }
            }

            GameState::Shop => {
                let action = ShopModal::draw(
                    &catalog,
                    save_data.stardust,
                    billing.is_ad_removed(),
                    &save_data.equipped_skin,
                    &save_data.unlocked_skins,
                    shop_status_msg.as_deref(),
                    effective_pointer,
                    mouse_clicked,
                );

                match action {
                    ShopAction::BuyItem(sku) => {
                        if sku.contains("removeads") || sku.contains("starpass") || sku.starts_with("com.gravipop.stardust") {
                            match billing.purchase_product(&sku) {
                                Ok(msg) => {
                                    shop_status_msg = Some(msg);
                                    if sku.contains("removeads") {
                                        save_data.ads_removed = true;
                                    }
                                    if sku.contains("starpass") {
                                        save_data.celestial_pass_unlocked = true;
                                    }
                                    if sku.ends_with(".500") {
                                        save_data.stardust += 500;
                                    } else if sku.ends_with(".2000") {
                                        save_data.stardust += 2000;
                                    }
                                    let _ = save_mgr.save(&save_data);
                                }
                                Err(err) => {
                                    shop_status_msg = Some(format!("Purchase Failed: {}", err));
                                }
                            }
                        } else {
                            if let Some(item) = catalog.skin_items.iter().find(|i| i.id == sku) {
                                if save_data.stardust >= item.stardust_price && !save_data.unlocked_skins.contains(&sku) {
                                    save_data.stardust -= item.stardust_price;
                                    save_data.unlocked_skins.push(sku.clone());
                                    save_data.equipped_skin = sku.clone();
                                    let _ = save_mgr.save(&save_data);
                                    shop_status_msg = Some(format!("Unlocked & Equipped {}!", item.title));
                                }
                            }
                        }
                    }
                    ShopAction::EquipSkin(skin_id) => {
                        save_data.equipped_skin = skin_id.clone();
                        let _ = save_mgr.save(&save_data);
                        shop_status_msg = Some(format!("Equipped {} theme!", skin_id));
                    }
                    ShopAction::Close => {
                        shop_status_msg = None;
                        if current_score > 0 {
                            game_state = GameState::GameOver;
                        } else {
                            game_state = GameState::MainMenu;
                        }
                    }
                    ShopAction::None => {}
                }
            }

            GameState::WatchingAd => {
                let reward_desc = match ads.get_ad_time_remaining() {
                    _ if revives_used == 0 && critical_timer >= CRITICAL_TIME_LIMIT => "Singularity Rewind & Revive",
                    _ if !stardust_doubled && run_stardust > 0 => "2x Cosmic Stardust Multiplier",
                    _ => "Cosmic Stardust Bounty",
                };

                let ad_result = MockAdOverlay::draw(
                    ads.get_ad_time_remaining(),
                    5.0,
                    reward_desc,
                    effective_pointer,
                    mouse_clicked,
                );

                match ad_result {
                    MockAdResult::ClaimReward => {
                        if let Some(reward) = ads.skip_or_finish() {
                            handle_reward(
                                reward,
                                &mut current_score,
                                &mut run_stardust,
                                &mut revives_used,
                                &mut stardust_doubled,
                                &mut bodies,
                                &mut critical_timer,
                                &mut save_data,
                                &save_mgr,
                                &mut game_state,
                            );
                        }
                    }
                    MockAdResult::CancelEarly => {
                        ads.skip_or_finish();
                        game_state = GameState::GameOver;
                    }
                    MockAdResult::None => {}
                }
            }
        }

        // =====================================================================
        // 5. DRAW VIRTUAL BUFFER TO PHYSICAL SCREEN (Auto-Scaled & Letterboxed)
        // =====================================================================
        set_default_camera();
        clear_background(Color::new(0.02, 0.02, 0.06, 1.0));

        draw_texture_ex(
            &virtual_target.texture,
            offset_x,
            offset_y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(VIRTUAL_WIDTH * scale, VIRTUAL_HEIGHT * scale)),
                flip_y: true, // Corrects OpenGL texture memory orientation
                ..Default::default()
            },
        );



        next_frame().await;
    }
}

fn handle_reward(
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
            bodies.sort_by(|a, b| b.pos.y.partial_cmp(&a.pos.y).unwrap_or(std::cmp::Ordering::Equal));
            let remove_count = (bodies.len() / 3).max(1);
            bodies.truncate(bodies.len().saturating_sub(remove_count));
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

fn is_in_rect(pos: Vec2, x: f32, y: f32, w: f32, h: f32) -> bool {
    pos.x >= x && pos.x <= x + w && pos.y >= y && pos.y <= y + h
}
