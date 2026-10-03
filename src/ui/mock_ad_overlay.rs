use macroquad::prelude::*;
use crate::core::config::{VIRTUAL_HEIGHT, VIRTUAL_WIDTH};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MockAdResult {
    None,
    ClaimReward,
    CancelEarly,
}

pub struct MockAdOverlay;

impl MockAdOverlay {
    pub fn draw(
        time_remaining: f32,
        total_duration: f32,
        reward_desc: &str,
        mouse_pos: Vec2,
        mouse_clicked: bool,
    ) -> MockAdResult {
        let mut result = MockAdResult::None;

        // Dark opaque ad backdrop
        draw_rectangle(0.0, 0.0, VIRTUAL_WIDTH, VIRTUAL_HEIGHT, Color::new(0.02, 0.02, 0.05, 0.98));

        // Ad Container Frame
        let frame_x = 25.0;
        let frame_y = 120.0;
        let frame_w = VIRTUAL_WIDTH - 50.0;
        let frame_h = 700.0;

        draw_rectangle(frame_x, frame_y, frame_w, frame_h, Color::new(0.07, 0.06, 0.15, 1.0));
        draw_rectangle_lines(frame_x, frame_y, frame_w, frame_h, 2.0, Color::new(0.9, 0.4, 0.1, 0.7));

        // Top Header
        draw_text("TEST AD NETWORK [SIMULATOR]", frame_x + 30.0, frame_y + 45.0, 20.0, Color::new(0.9, 0.7, 0.2, 1.0));
        let timer_label = format!("Reward in: {:.1}s", time_remaining.max(0.0));
        draw_text(&timer_label, frame_x + frame_w - 200.0, frame_y + 45.0, 20.0, WHITE);

        // Progress Bar
        let progress = ((total_duration - time_remaining) / total_duration).clamp(0.0, 1.0);
        let bar_x = frame_x + 30.0;
        let bar_y = frame_y + 65.0;
        let bar_w = frame_w - 60.0;
        let bar_h = 10.0;
        draw_rectangle(bar_x, bar_y, bar_w, bar_h, Color::new(0.2, 0.2, 0.3, 0.8));
        draw_rectangle(bar_x, bar_y, bar_w * progress, bar_h, Color::new(0.3, 0.9, 0.4, 1.0));

        // Ad Creative Mock Visuals
        let center_x = frame_x + frame_w * 0.5;
        let center_y = frame_y + 260.0;
        draw_circle(center_x, center_y, 110.0, Color::new(0.2, 0.1, 0.4, 0.8));
        draw_circle(center_x, center_y, 80.0, Color::new(0.9, 0.4, 0.1, 0.9));
        draw_text("GRAVIPOP", center_x - 70.0, center_y + 8.0, 30.0, WHITE);

        draw_text("NO ADS? NO PROBLEM!", frame_x + 85.0, frame_y + 430.0, 24.0, WHITE);
        draw_text("Support indie game development by watching or purchasing!", frame_x + 40.0, frame_y + 465.0, 15.0, Color::new(0.8, 0.8, 0.9, 0.8));

        let reward_label = format!("Target Reward: {}", reward_desc);
        draw_text(&reward_label, frame_x + 40.0, frame_y + 510.0, 18.0, Color::new(0.4, 1.0, 0.6, 1.0));

        // Claim / Finish Button (Always enabled in mock mode for snappy testing!)
        let btn_w = frame_w - 80.0;
        let btn_h = 58.0;
        let btn_x = frame_x + 40.0;
        let btn_y = frame_y + 580.0;

        let btn_hover = is_inside(mouse_pos, btn_x, btn_y, btn_w, btn_h);
        let btn_col = if btn_hover { Color::new(0.2, 0.85, 0.4, 1.0) } else { Color::new(0.15, 0.7, 0.3, 1.0) };
        draw_rectangle(btn_x, btn_y, btn_w, btn_h, btn_col);
        draw_rectangle_lines(btn_x, btn_y, btn_w, btn_h, 1.5, WHITE);

        let btn_txt = if time_remaining <= 0.0 { "CLAIM REWARD" } else { "SKIP & CLAIM REWARD [TEST]" };
        draw_text(btn_txt, btn_x + 45.0, btn_y + 38.0, 22.0, WHITE);

        if btn_hover && mouse_clicked {
            result = MockAdResult::ClaimReward;
        }

        // Cancel Early button
        let cancel_w = 120.0;
        let cancel_h = 36.0;
        let cancel_x = frame_x + frame_w * 0.5 - cancel_w * 0.5;
        let cancel_y = frame_y + 655.0;
        let cancel_hover = is_inside(mouse_pos, cancel_x, cancel_y, cancel_w, cancel_h);
        draw_text("Cancel Ad (No Reward)", cancel_x - 10.0, cancel_y + 24.0, 14.0, if cancel_hover { WHITE } else { Color::new(0.6, 0.6, 0.7, 0.8) });

        if cancel_hover && mouse_clicked {
            result = MockAdResult::CancelEarly;
        }

        result
    }
}

fn is_inside(pos: Vec2, x: f32, y: f32, w: f32, h: f32) -> bool {
    pos.x >= x && pos.x <= x + w && pos.y >= y && pos.y <= y + h
}
