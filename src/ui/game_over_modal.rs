use macroquad::prelude::*;
use crate::core::config::{VIRTUAL_HEIGHT, VIRTUAL_WIDTH};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameOverAction {
    None,
    WatchAdRevive,
    WatchAdDoubleStardust,
    Restart,
    OpenShop,
}

pub struct GameOverModal;

impl GameOverModal {
    pub fn draw(
        score: u64,
        high_score: u64,
        stardust_earned: u64,
        revive_available: bool,
        stardust_doubled: bool,
        mouse_pos: Vec2,
        mouse_clicked: bool,
    ) -> GameOverAction {
        let mut action = GameOverAction::None;

        // Dark dim backdrop
        draw_rectangle(0.0, 0.0, VIRTUAL_WIDTH, VIRTUAL_HEIGHT, Color::new(0.0, 0.0, 0.0, 0.75));

        // Center Glassmorphic Card
        let card_w = VIRTUAL_WIDTH - 60.0;
        let card_h = 580.0;
        let card_x = 30.0;
        let card_y = (VIRTUAL_HEIGHT - card_h) * 0.5 - 20.0;

        draw_rectangle(card_x, card_y, card_w, card_h, Color::new(0.08, 0.07, 0.16, 0.95));
        draw_rectangle_lines(card_x, card_y, card_w, card_h, 2.0, Color::new(0.85, 0.35, 1.0, 0.65));

        // Title
        draw_text("SINGULARITY COLLAPSED", card_x + 35.0, card_y + 55.0, 28.0, Color::new(1.0, 0.35, 0.45, 1.0));

        // Run Statistics Plaque
        let plaque_y = card_y + 85.0;
        draw_rectangle(card_x + 20.0, plaque_y, card_w - 40.0, 140.0, Color::new(0.12, 0.10, 0.22, 0.8));
        draw_rectangle_lines(card_x + 20.0, plaque_y, card_w - 40.0, 140.0, 1.0, Color::new(0.4, 0.35, 0.6, 0.4));

        draw_text("FINAL SCORE", card_x + 40.0, plaque_y + 35.0, 18.0, Color::new(0.7, 0.75, 0.9, 0.8));
        let score_txt = format!("{}", score);
        draw_text(&score_txt, card_x + 40.0, plaque_y + 70.0, 36.0, WHITE);

        if score >= high_score && score > 0 {
            draw_text("★ NEW HIGH SCORE! ★", card_x + 40.0, plaque_y + 98.0, 16.0, Color::new(1.0, 0.85, 0.2, 1.0));
        } else {
            let high_txt = format!("BEST: {}", high_score);
            draw_text(&high_txt, card_x + 40.0, plaque_y + 98.0, 16.0, Color::new(0.7, 0.7, 0.8, 0.7));
        }

        let dust_label = if stardust_doubled {
            format!("STARDUST: +{} (DOUBLED!)", stardust_earned)
        } else {
            format!("STARDUST: +{}", stardust_earned)
        };
        draw_text(&dust_label, card_x + 40.0, plaque_y + 125.0, 18.0, Color::new(0.4, 0.9, 1.0, 1.0));

        // Button 1: Rewarded Ad Revive
        let btn1_y = card_y + 245.0;
        let btn_h = 58.0;
        let btn_w = card_w - 40.0;
        let btn_x = card_x + 20.0;

        if revive_available {
            let hovered = is_inside(mouse_pos, btn_x, btn1_y, btn_w, btn_h);
            let btn_color = if hovered {
                Color::new(0.3, 0.75, 0.45, 1.0)
            } else {
                Color::new(0.2, 0.6, 0.35, 1.0)
            };
            draw_rectangle(btn_x, btn1_y, btn_w, btn_h, btn_color);
            draw_rectangle_lines(btn_x, btn1_y, btn_w, btn_h, 1.5, WHITE);
            draw_text("▶ WATCH AD: REWIND & REVIVE", btn_x + 35.0, btn1_y + 38.0, 20.0, WHITE);

            if hovered && mouse_clicked {
                action = GameOverAction::WatchAdRevive;
            }
        } else {
            draw_rectangle(btn_x, btn1_y, btn_w, btn_h, Color::new(0.2, 0.2, 0.25, 0.5));
            draw_text("REVIVE ALREADY USED", btn_x + 65.0, btn1_y + 38.0, 18.0, Color::new(0.5, 0.5, 0.55, 1.0));
        }

        // Button 2: Rewarded Ad 2x Stardust
        let btn2_y = card_y + 318.0;
        if !stardust_doubled && stardust_earned > 0 {
            let hovered = is_inside(mouse_pos, btn_x, btn2_y, btn_w, btn_h);
            let btn_color = if hovered {
                Color::new(0.85, 0.55, 0.15, 1.0)
            } else {
                Color::new(0.75, 0.45, 0.1, 1.0)
            };
            draw_rectangle(btn_x, btn2_y, btn_w, btn_h, btn_color);
            draw_rectangle_lines(btn_x, btn2_y, btn_w, btn_h, 1.5, WHITE);
            draw_text("★ WATCH AD: 2X STARDUST", btn_x + 48.0, btn2_y + 38.0, 20.0, WHITE);

            if hovered && mouse_clicked {
                action = GameOverAction::WatchAdDoubleStardust;
            }
        }

        // Button 3: Retry
        let btn3_y = card_y + 395.0;
        let hovered3 = is_inside(mouse_pos, btn_x, btn3_y, btn_w, btn_h);
        let btn3_color = if hovered3 {
            Color::new(0.3, 0.45, 0.85, 1.0)
        } else {
            Color::new(0.2, 0.35, 0.7, 1.0)
        };
        draw_rectangle(btn_x, btn3_y, btn_w, btn_h, btn3_color);
        draw_rectangle_lines(btn_x, btn3_y, btn_w, btn_h, 1.5, WHITE);
        draw_text("↺ PLAY AGAIN", btn_x + 115.0, btn3_y + 38.0, 22.0, WHITE);

        if hovered3 && mouse_clicked {
            action = GameOverAction::Restart;
        }

        // Button 4: Shop
        let btn4_y = card_y + 470.0;
        let hovered4 = is_inside(mouse_pos, btn_x, btn4_y, btn_w, btn_h);
        let btn4_color = if hovered4 {
            Color::new(0.6, 0.25, 0.75, 1.0)
        } else {
            Color::new(0.45, 0.18, 0.6, 1.0)
        };
        draw_rectangle(btn_x, btn4_y, btn_w, btn_h, btn4_color);
        draw_rectangle_lines(btn_x, btn4_y, btn_w, btn_h, 1.5, WHITE);
        draw_text("✦ COSMIC OUTPOST (SHOP)", btn_x + 55.0, btn4_y + 38.0, 20.0, WHITE);

        if hovered4 && mouse_clicked {
            action = GameOverAction::OpenShop;
        }

        action
    }
}

fn is_inside(pos: Vec2, x: f32, y: f32, w: f32, h: f32) -> bool {
    pos.x >= x && pos.x <= x + w && pos.y >= y && pos.y <= y + h
}
