use macroquad::prelude::*;
use crate::core::config::{CRITICAL_TIME_LIMIT, VIRTUAL_WIDTH};
use crate::physics::CelestialTier;

pub struct Hud;

fn dtx(text: &str, x: f32, y: f32, sz: f32, col: Color, font: Option<&Font>) {
    draw_text_ex(text, x, y, TextParams { font, font_size: sz as u16, color: col, ..Default::default() });
}

fn dcx(text: &str, cx: f32, y: f32, sz: f32, col: Color, font: Option<&Font>) {
    let dim = measure_text(text, font, sz as u16, 1.0);
    draw_text_ex(text, cx - dim.width * 0.5, y, TextParams { font, font_size: sz as u16, color: col, ..Default::default() });
}

impl Hud {
    #[allow(clippy::too_many_arguments)]
    pub fn draw(
        score: u64,
        high_score: u64,
        stardust: u64,
        combo: usize,
        combo_timer: f32,
        critical_timer: f32,
        next_tier: CelestialTier,
        font: Option<&Font>,
    ) {
        let panel_h = 105.0;

        // ── Top HUD Panel ────────────────────────────────────────────────────
        draw_rectangle(
            10.0, 10.0, VIRTUAL_WIDTH - 20.0, panel_h,
            Color::new(0.07, 0.06, 0.16, 0.82),
        );
        draw_rectangle_lines(
            10.0, 10.0, VIRTUAL_WIDTH - 20.0, panel_h,
            1.5, Color::new(0.40, 0.35, 0.70, 0.50),
        );

        // SCORE (left column)
        dtx("SCORE", 28.0, 46.0, 20.0, Color::new(0.70, 0.75, 0.90, 0.80), font);
        let sc = score.to_string();
        dtx(&sc, 28.0, 94.0, 46.0, WHITE, font);

        // BEST (right of score)
        let best = format!("BEST  {}", high_score);
        dtx(&best, 260.0, 46.0, 20.0, Color::new(0.95, 0.82, 0.30, 0.90), font);

        let dust = format!("DUST  {}", stardust);
        dtx(&dust, 260.0, 80.0, 20.0, Color::new(0.42, 0.90, 1.00, 1.00), font);

        // NEXT preview (far right)
        let prev_cx = VIRTUAL_WIDTH - 46.0;
        let prev_cy = 60.0;
        dtx("NEXT", prev_cx - 52.0, 34.0, 18.0, Color::new(0.70, 0.75, 0.90, 0.75), font);
        draw_circle(prev_cx, prev_cy, next_tier.radius() * 0.80, next_tier.primary_color());
        draw_circle_lines(prev_cx, prev_cy, next_tier.radius() * 0.80, 1.5, WHITE);

        // ── Combo notification ────────────────────────────────────────────────
        if combo > 1 && combo_timer > 0.0 {
            let alpha = (combo_timer / 2.2).clamp(0.0, 1.0);
            let txt = format!("{}x COMBO!", combo);
            dcx(
                &txt, VIRTUAL_WIDTH * 0.5, 152.0, 38.0,
                Color::new(1.0, 0.88, 0.22, alpha), font,
            );
        }

        // ── Event Horizon danger banner ───────────────────────────────────────
        if critical_timer > 0.0 {
            let secs_left = (CRITICAL_TIME_LIMIT - critical_timer).max(0.0);
            let pulse = (get_time() as f32 * 14.0).sin().abs();
            let r = Color::new(0.88, 0.14, 0.14, 0.78 + pulse * 0.22);

            draw_rectangle(20.0, 170.0, VIRTUAL_WIDTH - 40.0, 56.0, r);
            draw_rectangle_lines(20.0, 170.0, VIRTUAL_WIDTH - 40.0, 56.0, 2.0, WHITE);

            let warn = format!("CRITICAL OVERFLOW  {:.1}s", secs_left);
            dcx(&warn, VIRTUAL_WIDTH * 0.5, 208.0, 24.0, WHITE, font);
        }
    }
}
