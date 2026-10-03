use macroquad::prelude::*;
use crate::core::config::{CRITICAL_TIME_LIMIT, VIRTUAL_WIDTH};
use crate::physics::CelestialTier;

pub struct Hud;

impl Hud {
    pub fn draw(
        score: u64,
        high_score: u64,
        stardust: u64,
        combo: usize,
        combo_timer: f32,
        critical_timer: f32,
        next_tier: CelestialTier,
    ) {
        // 1. Top Glassmorphic Panel
        draw_rectangle(15.0, 15.0, VIRTUAL_WIDTH - 30.0, 75.0, Color::new(0.08, 0.07, 0.16, 0.75));
        draw_rectangle_lines(15.0, 15.0, VIRTUAL_WIDTH - 30.0, 75.0, 1.5, Color::new(0.4, 0.35, 0.7, 0.45));

        // Score display
        let score_str = format!("{}", score);
        draw_text("SCORE", 35.0, 42.0, 18.0, Color::new(0.7, 0.75, 0.9, 0.8));
        draw_text(&score_str, 35.0, 75.0, 32.0, WHITE);

        // Best score
        let best_str = format!("BEST: {}", high_score);
        draw_text(&best_str, 250.0, 42.0, 16.0, Color::new(0.95, 0.8, 0.3, 0.85));

        // Stardust Currency
        let dust_str = format!("✦ {}", stardust);
        draw_text(&dust_str, 250.0, 75.0, 24.0, Color::new(0.4, 0.9, 1.0, 0.95));

        // Next queued planet preview box
        let preview_x = VIRTUAL_WIDTH - 100.0;
        let preview_y = 52.0;
        draw_text("NEXT", preview_x - 45.0, 58.0, 16.0, Color::new(0.7, 0.75, 0.9, 0.7));
        draw_circle(preview_x + 10.0, preview_y, next_tier.radius() * 0.75, next_tier.primary_color());
        draw_circle_lines(preview_x + 10.0, preview_y, next_tier.radius() * 0.75, 1.2, WHITE);

        // 2. Combo Counter Notification
        if combo > 1 && combo_timer > 0.0 {
            let combo_str = format!("{}x COMBO!", combo);
            let alpha = (combo_timer / 2.2).clamp(0.0, 1.0);
            let combo_color = Color::new(1.0, 0.85, 0.2, alpha);
            draw_text(&combo_str, VIRTUAL_WIDTH * 0.5 - 75.0, 130.0, 32.0, combo_color);
        }

        // 3. Event Horizon Danger Warning Banner
        if critical_timer > 0.0 {
            let time_left = (CRITICAL_TIME_LIMIT - critical_timer).max(0.0);
            let pulse = (get_time() as f32 * 14.0).sin().abs();
            let banner_color = Color::new(0.85, 0.15, 0.15, 0.75 + pulse * 0.25);

            draw_rectangle(30.0, 155.0, VIRTUAL_WIDTH - 60.0, 48.0, banner_color);
            draw_rectangle_lines(30.0, 155.0, VIRTUAL_WIDTH - 60.0, 48.0, 2.0, WHITE);

            let warn_text = format!("⚠️ EVENT HORIZON OVERFLOW: {:.1}s", time_left);
            draw_text(&warn_text, 55.0, 187.0, 22.0, WHITE);
        }
    }
}
