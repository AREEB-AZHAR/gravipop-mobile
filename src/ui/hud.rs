use crate::core::config::{CRITICAL_TIME_LIMIT, VIRTUAL_WIDTH};
use crate::graphics::icons::{draw_vector_gem, draw_vector_pause};
use macroquad::prelude::*;

pub struct Hud;

impl Hud {
    /// Draw the compact endless-run top bar.
    pub fn draw(
        score: u64,
        high_score: u64,
        stardust: u64,
        danger_timer: f32,
        font: Option<&Font>,
    ) {
        let cx = VIRTUAL_WIDTH * 0.5;
        let f = font;
        let sz = |n: f32| n as u16;

        let center_txt = |text: &str, x: f32, y: f32, size: f32, color: Color| {
            let dim = measure_text(text, f, sz(size), 1.0);
            draw_text_ex(
                text,
                x - dim.width * 0.5,
                y,
                TextParams {
                    font: f,
                    font_size: sz(size),
                    color,
                    ..Default::default()
                },
            );
        };
        let left_txt = |text: &str, x: f32, y: f32, size: f32, color: Color| {
            draw_text_ex(
                text,
                x,
                y,
                TextParams {
                    font: f,
                    font_size: sz(size),
                    color,
                    ..Default::default()
                },
            );
        };
        // ── 1. Top Bar (y: 15..85) ───────────────────────────────────────────

        // Stardust Badge (Top Left)
        let dust_card_x = 20.0;
        let dust_card_y = 20.0;
        let dust_card_w = 150.0;
        let dust_card_h = 48.0;
        draw_rectangle(
            dust_card_x,
            dust_card_y,
            dust_card_w,
            dust_card_h,
            Color::new(0.08, 0.08, 0.18, 0.85),
        );
        draw_rectangle_lines(
            dust_card_x,
            dust_card_y,
            dust_card_w,
            dust_card_h,
            1.5,
            Color::new(0.35, 0.55, 0.90, 0.55),
        );
        draw_vector_gem(
            dust_card_x + 24.0,
            dust_card_y + 24.0,
            20.0,
            Color::new(0.30, 0.85, 1.0, 1.0),
        );
        left_txt(
            &format!("{}", stardust),
            dust_card_x + 44.0,
            dust_card_y + 32.0,
            22.0,
            Color::new(0.85, 0.95, 1.0, 1.0),
        );

        // Score Pill (Top Center)
        let score_str = format!("{}", score);
        center_txt(&score_str, cx, 52.0, 48.0, WHITE);
        let best_str = format!("BEST {}", high_score);
        center_txt(&best_str, cx, 74.0, 18.0, Color::new(1.0, 0.84, 0.30, 0.90));

        // Pause Button (Top Right)
        let pause_x = VIRTUAL_WIDTH - 80.0;
        let pause_y = 20.0;
        let pause_w = 60.0;
        let pause_h = 48.0;
        draw_rectangle(
            pause_x,
            pause_y,
            pause_w,
            pause_h,
            Color::new(0.08, 0.08, 0.18, 0.85),
        );
        draw_rectangle_lines(
            pause_x,
            pause_y,
            pause_w,
            pause_h,
            1.5,
            Color::new(0.45, 0.45, 0.70, 0.55),
        );
        draw_vector_pause(
            pause_x + pause_w * 0.5,
            pause_y + pause_h * 0.5,
            20.0,
            20.0,
            WHITE,
        );

        // ── 2. Endless-run label ──────────────────────────────────────────────
        center_txt(
            "ENDLESS ORBIT",
            cx,
            105.0,
            15.0,
            Color::new(0.56, 0.78, 1.0, 0.90),
        );

        // ── 3. Danger Warning Banner (at top edge) ────────────────────────────
        if danger_timer > 0.3 {
            let t = (danger_timer / CRITICAL_TIME_LIMIT).clamp(0.0, 1.0);
            let pulse = (get_time() as f32 * 14.0).sin().abs();
            let alpha = 0.45 + t * 0.55 * pulse;
            draw_rectangle(
                0.0,
                0.0,
                VIRTUAL_WIDTH,
                8.0,
                Color::new(1.0, 0.20, 0.25, alpha),
            );
        }
    }
}
