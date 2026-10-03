use macroquad::prelude::*;
use crate::core::config::{VIRTUAL_HEIGHT, VIRTUAL_WIDTH};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MockAdResult {
    None,
    ClaimReward,
    CancelEarly,
}

pub struct MockAdOverlay;

fn dcx(text: &str, cx: f32, y: f32, sz: f32, col: Color, font: Option<&Font>) {
    let dim = measure_text(text, font, sz as u16, 1.0);
    draw_text_ex(text, cx - dim.width * 0.5, y, TextParams { font, font_size: sz as u16, color: col, ..Default::default() });
}

fn dtx(text: &str, x: f32, y: f32, sz: f32, col: Color, font: Option<&Font>) {
    draw_text_ex(text, x, y, TextParams { font, font_size: sz as u16, color: col, ..Default::default() });
}

fn inside(p: Vec2, x: f32, y: f32, w: f32, h: f32) -> bool {
    p.x >= x && p.x <= x + w && p.y >= y && p.y <= y + h
}

impl MockAdOverlay {
    pub fn draw(
        time_remaining: f32,
        total_duration: f32,
        reward_desc: &str,
        mouse_pos: Vec2,
        mouse_clicked: bool,
        font: Option<&Font>,
    ) -> MockAdResult {
        let mut result = MockAdResult::None;
        let cx = VIRTUAL_WIDTH * 0.5;

        // Full-screen backdrop
        draw_rectangle(0.0, 0.0, VIRTUAL_WIDTH, VIRTUAL_HEIGHT, Color::new(0.02, 0.02, 0.06, 0.98));

        // Ad frame
        let fx = 20.0;
        let fy = 100.0;
        let fw = VIRTUAL_WIDTH - 40.0;
        let fh = 750.0;
        draw_rectangle(fx, fy, fw, fh, Color::new(0.07, 0.06, 0.16, 1.0));
        draw_rectangle_lines(fx, fy, fw, fh, 2.0, Color::new(0.92, 0.42, 0.10, 0.75));

        // Header
        dcx("AD SIMULATION", cx, fy + 40.0, 22.0, Color::new(0.92, 0.72, 0.22, 1.0), font);
        let timer_str = format!("Reward in  {:.1}s", time_remaining.max(0.0));
        dcx(&timer_str, cx, fy + 72.0, 22.0, WHITE, font);

        // Progress bar
        let progress = ((total_duration - time_remaining) / total_duration).clamp(0.0, 1.0);
        let bx = fx + 28.0;
        let by = fy + 86.0;
        let bw = fw - 56.0;
        draw_rectangle(bx, by, bw, 12.0, Color::new(0.18, 0.18, 0.28, 0.85));
        draw_rectangle(bx, by, bw * progress, 12.0, Color::new(0.30, 0.92, 0.42, 1.0));

        // Decorative mock creative
        let mid_y = fy + 290.0;
        draw_circle(cx, mid_y, 115.0, Color::new(0.18, 0.10, 0.40, 0.85));
        draw_circle(cx, mid_y, 82.0, Color::new(0.90, 0.42, 0.10, 0.92));
        dcx("GRAVIPOP", cx, mid_y + 12.0, 32.0, WHITE, font);

        dcx("SUPPORT INDIE GAMES!", cx, fy + 460.0, 26.0, WHITE, font);
        dcx("Watch ads or purchase to remove them.", cx, fy + 500.0, 18.0, Color::new(0.80, 0.80, 0.90, 0.82), font);

        let rd = format!("Reward:  {}", reward_desc);
        dcx(&rd, cx, fy + 545.0, 20.0, Color::new(0.42, 1.0, 0.62, 1.0), font);

        // Claim button
        let cbx = fx + 36.0;
        let cbw = fw - 72.0;
        let cby = fy + 590.0;
        let cbh = 66.0;
        let chov = inside(mouse_pos, cbx, cby, cbw, cbh);
        draw_rectangle(cbx, cby, cbw, cbh,
            if chov { Color::new(0.22, 0.88, 0.44, 1.0) } else { Color::new(0.16, 0.72, 0.32, 1.0) });
        draw_rectangle_lines(cbx, cby, cbw, cbh, 1.8, WHITE);
        let btn_label = if time_remaining <= 0.0 { "CLAIM REWARD" } else { "SKIP & CLAIM [TEST MODE]" };
        dcx(btn_label, cx, cby + 43.0, 24.0, WHITE, font);
        if chov && mouse_clicked { result = MockAdResult::ClaimReward; }

        // Cancel link
        let cancel_y = cby + cbh + 32.0;
        let cancel_hov = inside(mouse_pos, fx + 60.0, cancel_y - 24.0, fw - 120.0, 36.0);
        dcx("Cancel Ad (No Reward)", cx, cancel_y, 18.0,
            if cancel_hov { WHITE } else { Color::new(0.60, 0.60, 0.72, 0.80) }, font);
        if cancel_hov && mouse_clicked { result = MockAdResult::CancelEarly; }

        // Timer overlay badge top-right
        let badge_str = format!("{:.0}s", time_remaining.max(0.0).ceil());
        dtx(&badge_str, fx + fw - 60.0, fy + 32.0, 22.0, Color::new(1.0, 0.88, 0.22, 1.0), font);

        result
    }
}
