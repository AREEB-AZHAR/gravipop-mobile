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

        let center_txt = |text: &str, x: f32, y: f32, size: f32, color: Color| {
            let raster_size = (size * 2.0).round().max(16.0) as u16;
            let font_scale = 0.5;
            let dim = measure_text(text, f, raster_size, font_scale);
            let px = (x - dim.width * 0.5).round();
            let py = y.round();
            draw_text_ex(
                text,
                px + 1.2,
                py + 1.2,
                TextParams {
                    font: f,
                    font_size: raster_size,
                    font_scale,
                    color: Color::new(0.0, 0.0, 0.0, 0.85),
                    ..Default::default()
                },
            );
            draw_text_ex(
                text,
                px + 0.65,
                py,
                TextParams {
                    font: f,
                    font_size: raster_size,
                    font_scale,
                    color,
                    ..Default::default()
                },
            );
            draw_text_ex(
                text,
                px,
                py,
                TextParams {
                    font: f,
                    font_size: raster_size,
                    font_scale,
                    color,
                    ..Default::default()
                },
            );
        };
        let left_txt = |text: &str, x: f32, y: f32, size: f32, color: Color| {
            let raster_size = (size * 2.0).round().max(16.0) as u16;
            let font_scale = 0.5;
            let px = x.round();
            let py = y.round();
            draw_text_ex(
                text,
                px + 1.2,
                py + 1.2,
                TextParams {
                    font: f,
                    font_size: raster_size,
                    font_scale,
                    color: Color::new(0.0, 0.0, 0.0, 0.85),
                    ..Default::default()
                },
            );
            draw_text_ex(
                text,
                px + 0.65,
                py,
                TextParams {
                    font: f,
                    font_size: raster_size,
                    font_scale,
                    color,
                    ..Default::default()
                },
            );
            draw_text_ex(
                text,
                px,
                py,
                TextParams {
                    font: f,
                    font_size: raster_size,
                    font_scale,
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

        // ── 3. Danger Warning Banner & Mid-Screen Countdown ───────────────────
        if danger_timer > 0.05 {
            let t = (danger_timer / CRITICAL_TIME_LIMIT).clamp(0.0, 1.0);
            let pulse = (get_time() as f32 * 10.0).sin().abs();
            let alpha = 0.45 + t * 0.55 * pulse;
            draw_rectangle(
                0.0,
                0.0,
                VIRTUAL_WIDTH,
                8.0,
                Color::new(1.0, 0.20, 0.25, alpha),
            );

            // Mid-Screen 5.0s Overflow Countdown Timer
            let remaining = (CRITICAL_TIME_LIMIT - danger_timer).max(0.0);
            let box_w = 340.0;
            let box_h = 58.0;
            let box_x = cx - box_w * 0.5;
            let box_y = 365.0;

            // Semi-transparent dark alert backdrop with pulsing neon red border
            let bg_color = Color::new(0.18 + 0.12 * pulse, 0.02, 0.04, 0.92);
            let border_color = Color::new(1.0, 0.25 + 0.35 * pulse, 0.30, 0.98);
            draw_rectangle(box_x, box_y, box_w, box_h, bg_color);
            draw_rectangle_lines(box_x - 1.5, box_y - 1.5, box_w + 3.0, box_h + 3.0, 1.0, Color::new(1.0, 0.20, 0.25, 0.35 + 0.25 * pulse));
            draw_rectangle_lines(box_x, box_y, box_w, box_h, 2.5, border_color);

            let timer_str = format!("OVERFLOW IN {:.1}s", remaining);
            center_txt(
                &timer_str,
                cx,
                box_y + 30.0,
                23.0,
                Color::new(1.0, 0.94, 0.94, 1.0),
            );

            // Sleek animated progress bar draining with remaining countdown
            let bar_margin = 24.0;
            let bar_w = box_w - bar_margin * 2.0;
            let bar_h = 5.0;
            let bar_x = box_x + bar_margin;
            let bar_y = box_y + box_h - 11.0;
            let progress_ratio = (remaining / CRITICAL_TIME_LIMIT).clamp(0.0, 1.0);

            draw_rectangle(bar_x, bar_y, bar_w, bar_h, Color::new(0.28, 0.08, 0.10, 0.85));
            draw_rectangle(
                bar_x,
                bar_y,
                bar_w * progress_ratio,
                bar_h,
                Color::new(1.0, 0.30 + 0.30 * pulse, 0.35, 1.0),
            );
        }
    }
}
