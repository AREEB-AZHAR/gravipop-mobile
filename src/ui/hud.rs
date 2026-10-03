use macroquad::prelude::*;
use crate::core::config::{CRITICAL_TIME_LIMIT, VIRTUAL_WIDTH};
use crate::core::sector::ObjectiveTracker;
use crate::graphics::icons::{draw_vector_gem, draw_vector_pause};

pub struct Hud;

impl Hud {
    /// Draw top bar (score, best, stardust, pause) and sector objective banner
    #[allow(clippy::too_many_arguments)]
    pub fn draw(
        score: u64,
        high_score: u64,
        stardust: u64,
        sector_name: &str,
        sector_index: usize,
        objective: &ObjectiveTracker,
        danger_timer: f32,
        font: Option<&Font>,
    ) {
        let cx = VIRTUAL_WIDTH * 0.5;
        let f = font;
        let sz = |n: f32| n as u16;

        let center_txt = |text: &str, x: f32, y: f32, size: f32, color: Color| {
            let dim = measure_text(text, f, sz(size), 1.0);
            draw_text_ex(text, x - dim.width * 0.5, y,
                TextParams { font: f, font_size: sz(size), color, ..Default::default() });
        };
        let left_txt = |text: &str, x: f32, y: f32, size: f32, color: Color| {
            draw_text_ex(text, x, y,
                TextParams { font: f, font_size: sz(size), color, ..Default::default() });
        };
        let right_txt = |text: &str, x: f32, y: f32, size: f32, color: Color| {
            let dim = measure_text(text, f, sz(size), 1.0);
            draw_text_ex(text, x - dim.width, y,
                TextParams { font: f, font_size: sz(size), color, ..Default::default() });
        };

        // ── 1. Top Bar (y: 15..85) ───────────────────────────────────────────
        
        // Stardust Badge (Top Left)
        let dust_card_x = 20.0;
        let dust_card_y = 20.0;
        let dust_card_w = 150.0;
        let dust_card_h = 48.0;
        draw_rectangle(dust_card_x, dust_card_y, dust_card_w, dust_card_h, Color::new(0.08, 0.08, 0.18, 0.85));
        draw_rectangle_lines(dust_card_x, dust_card_y, dust_card_w, dust_card_h, 1.5, Color::new(0.35, 0.55, 0.90, 0.55));
        draw_vector_gem(dust_card_x + 24.0, dust_card_y + 24.0, 20.0, Color::new(0.30, 0.85, 1.0, 1.0));
        left_txt(&format!("{}", stardust), dust_card_x + 44.0, dust_card_y + 32.0, 22.0, Color::new(0.85, 0.95, 1.0, 1.0));

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
        draw_rectangle(pause_x, pause_y, pause_w, pause_h, Color::new(0.08, 0.08, 0.18, 0.85));
        draw_rectangle_lines(pause_x, pause_y, pause_w, pause_h, 1.5, Color::new(0.45, 0.45, 0.70, 0.55));
        draw_vector_pause(pause_x + pause_w * 0.5, pause_y + pause_h * 0.5, 20.0, 20.0, WHITE);

        // ── 2. Sector Objective Card (y: 95..175) ─────────────────────────────
        let obj_x = 90.0;
        let obj_y = 95.0;
        let obj_w = VIRTUAL_WIDTH - 180.0; // 540.0
        let obj_h = 76.0;

        // Glass background
        draw_rectangle(obj_x, obj_y, obj_w, obj_h, Color::new(0.06, 0.05, 0.16, 0.88));
        draw_rectangle_lines(obj_x, obj_y, obj_w, obj_h, 1.5, Color::new(0.40, 0.35, 0.80, 0.65));

        // Sector header
        let sec_title = format!("SECTOR {}: {}", sector_index + 1, sector_name);
        left_txt(&sec_title, obj_x + 16.0, obj_y + 24.0, 16.0, Color::new(0.55, 0.80, 1.0, 0.90));

        // Objective description
        let goal_desc = objective.objective.display_text();
        left_txt(&goal_desc, obj_x + 16.0, obj_y + 48.0, 20.0, WHITE);

        // Progress bar
        let progress = objective.progress_fraction();
        let progress_txt = objective.progress_text();
        let bar_x = obj_x + 16.0;
        let bar_y = obj_y + 58.0;
        let bar_w = obj_w - 32.0;
        let bar_h = 8.0;

        draw_rectangle(bar_x, bar_y, bar_w, bar_h, Color::new(0.14, 0.14, 0.25, 0.90));
        let fill_col = if progress >= 1.0 {
            Color::new(0.28, 0.95, 0.45, 1.0)
        } else {
            Color::new(0.35, 0.70, 1.0, 1.0)
        };
        draw_rectangle(bar_x, bar_y, (bar_w * progress).clamp(0.0, bar_w), bar_h, fill_col);

        // Progress fraction on right
        right_txt(&progress_txt, obj_x + obj_w - 16.0, obj_y + 48.0, 16.0, Color::new(0.80, 0.85, 0.95, 0.90));

        // ── 3. Danger Warning Banner (at top edge) ────────────────────────────
        if danger_timer > 0.3 {
            let t = (danger_timer / CRITICAL_TIME_LIMIT).clamp(0.0, 1.0);
            let pulse = (get_time() as f32 * 14.0).sin().abs();
            let alpha = 0.45 + t * 0.55 * pulse;
            draw_rectangle(0.0, 0.0, VIRTUAL_WIDTH, 8.0, Color::new(1.0, 0.20, 0.25, alpha));
        }
    }
}
