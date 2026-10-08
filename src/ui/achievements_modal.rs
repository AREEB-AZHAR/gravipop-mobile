use crate::core::achievements::{AchievementCategory, ALL_ACHIEVEMENTS};
use crate::core::config::{VIRTUAL_HEIGHT, VIRTUAL_WIDTH};
use crate::graphics::icons::draw_vector_close;
use macroquad::prelude::*;

#[derive(Debug, Clone)]
pub enum AchievementsAction {
    None,
    Close,
}

pub struct AchievementsModal;

fn dtx(text: &str, x: f32, y: f32, sz: f32, col: Color, font: Option<&Font>) {
    draw_text_ex(
        text,
        x,
        y,
        TextParams {
            font,
            font_size: sz as u16,
            color: col,
            ..Default::default()
        },
    );
}

fn dcx(text: &str, cx: f32, y: f32, sz: f32, col: Color, font: Option<&Font>) {
    let dim = measure_text(text, font, sz as u16, 1.0);
    draw_text_ex(
        text,
        cx - dim.width * 0.5,
        y,
        TextParams {
            font,
            font_size: sz as u16,
            color: col,
            ..Default::default()
        },
    );
}

fn inside(p: Vec2, x: f32, y: f32, w: f32, h: f32) -> bool {
    p.x >= x && p.x <= x + w && p.y >= y && p.y <= y + h
}

impl AchievementsModal {
    pub fn draw(
        unlocked_ids: &[String],
        current_score: u64,
        mouse_pos: Vec2,
        mouse_clicked: bool,
        font: Option<&Font>,
    ) -> AchievementsAction {
        let mut action = AchievementsAction::None;
        let cx = VIRTUAL_WIDTH * 0.5;

        // Dim backdrop
        draw_rectangle(
            0.0,
            0.0,
            VIRTUAL_WIDTH,
            VIRTUAL_HEIGHT,
            Color::new(0.0, 0.0, 0.0, 0.88),
        );

        // Glassmorphic Modal Container
        let cw = VIRTUAL_WIDTH - 36.0;
        let ch = VIRTUAL_HEIGHT - 120.0;
        let card_x = 18.0;
        let card_y = 60.0;

        draw_rectangle(card_x, card_y, cw, ch, Color::new(0.05, 0.06, 0.12, 0.98));
        draw_rectangle_lines(
            card_x,
            card_y,
            cw,
            ch,
            2.0,
            Color::new(0.95, 0.75, 0.25, 0.75),
        );

        // Header
        dcx("COSMIC ACHIEVEMENTS", cx, card_y + 46.0, 32.0, WHITE, font);

        let unlocked_count = ALL_ACHIEVEMENTS
            .iter()
            .filter(|a| unlocked_ids.iter().any(|id| id == a.id))
            .count();
        dcx(
            &format!(
                "UNLOCKED: {} / {} MILESTONES",
                unlocked_count,
                ALL_ACHIEVEMENTS.len()
            ),
            cx,
            card_y + 74.0,
            16.0,
            Color::new(1.0, 0.85, 0.40, 0.95),
            font,
        );

        // Close Button
        let close_btn_x = card_x + cw - 46.0;
        let close_btn_y = card_y + 14.0;
        let close_hov = inside(mouse_pos, close_btn_x, close_btn_y, 32.0, 32.0);
        let close_col = if close_hov {
            Color::new(1.0, 0.40, 0.40, 1.0)
        } else {
            Color::new(0.70, 0.70, 0.85, 0.80)
        };
        draw_vector_close(close_btn_x + 16.0, close_btn_y + 16.0, 10.0, 2.0, close_col);
        if (close_hov && mouse_clicked) || is_key_pressed(KeyCode::Escape) {
            action = AchievementsAction::Close;
        }

        // Scrollable / Paged List of Achievements
        let mut row_y = card_y + 104.0;
        let card_w = cw - 40.0;
        let card_h = 48.0;

        for ach in ALL_ACHIEVEMENTS.iter().take(18) {
            let is_unlocked = unlocked_ids.iter().any(|id| id == ach.id);
            let bg = if is_unlocked {
                Color::new(0.12, 0.18, 0.28, 0.90)
            } else {
                Color::new(0.06, 0.07, 0.13, 0.75)
            };

            let item_x = card_x + 20.0;
            draw_rectangle(item_x, row_y, card_w, card_h, bg);

            let border_col = if is_unlocked {
                Color::new(0.95, 0.80, 0.30, 0.80)
            } else {
                Color::new(0.20, 0.25, 0.35, 0.40)
            };
            draw_rectangle_lines(item_x, row_y, card_w, card_h, 1.2, border_col);

            // Icon / Pip
            let icon_x = item_x + 14.0;
            if is_unlocked {
                draw_circle(
                    icon_x + 10.0,
                    row_y + card_h * 0.5,
                    9.0,
                    Color::new(1.0, 0.82, 0.25, 1.0),
                );
                dcx(
                    "★",
                    icon_x + 10.0,
                    row_y + card_h * 0.5 + 5.0,
                    14.0,
                    BLACK,
                    font,
                );
            } else {
                draw_circle(
                    icon_x + 10.0,
                    row_y + card_h * 0.5,
                    9.0,
                    Color::new(0.18, 0.22, 0.30, 1.0),
                );
                dcx(
                    "•",
                    icon_x + 10.0,
                    row_y + card_h * 0.5 + 4.0,
                    14.0,
                    Color::new(0.5, 0.5, 0.6, 0.8),
                    font,
                );
            }

            // Title & Description
            let text_x = item_x + 44.0;
            dtx(
                ach.title,
                text_x,
                row_y + 20.0,
                16.0,
                if is_unlocked {
                    WHITE
                } else {
                    Color::new(0.65, 0.70, 0.80, 0.85)
                },
                font,
            );
            dtx(
                ach.description,
                text_x,
                row_y + 38.0,
                12.0,
                if is_unlocked {
                    Color::new(0.80, 0.90, 1.0, 0.80)
                } else {
                    Color::new(0.45, 0.50, 0.60, 0.70)
                },
                font,
            );

            // Status Badge
            let badge_w = 78.0;
            let badge_x = item_x + card_w - badge_w - 10.0;
            let badge_y = row_y + (card_h - 24.0) * 0.5;

            if is_unlocked {
                draw_rectangle(
                    badge_x,
                    badge_y,
                    badge_w,
                    24.0,
                    Color::new(0.15, 0.65, 0.35, 0.95),
                );
                dcx(
                    "CLAIMED",
                    badge_x + badge_w * 0.5,
                    badge_y + 17.0,
                    12.0,
                    WHITE,
                    font,
                );
            } else {
                draw_rectangle(
                    badge_x,
                    badge_y,
                    badge_w,
                    24.0,
                    Color::new(0.14, 0.16, 0.24, 0.85),
                );
                if ach.category == AchievementCategory::Score {
                    let pct =
                        ((current_score as f32 / ach.target_value as f32).min(1.0) * 100.0) as u32;
                    dcx(
                        &format!("{}%", pct),
                        badge_x + badge_w * 0.5,
                        badge_y + 17.0,
                        12.0,
                        Color::new(0.65, 0.75, 0.90, 0.85),
                        font,
                    );
                } else {
                    dcx(
                        "LOCKED",
                        badge_x + badge_w * 0.5,
                        badge_y + 17.0,
                        12.0,
                        Color::new(0.55, 0.60, 0.70, 0.75),
                        font,
                    );
                }
            }

            row_y += card_h + 8.0;
        }

        // Close / Back button
        let done_w = 260.0;
        let done_h = 54.0;
        let done_x = cx - done_w * 0.5;
        let done_y = card_y + ch - 68.0;
        let done_hov = inside(mouse_pos, done_x, done_y, done_w, done_h);

        draw_rectangle(
            done_x,
            done_y,
            done_w,
            done_h,
            if done_hov {
                Color::new(0.95, 0.75, 0.25, 1.0)
            } else {
                Color::new(0.75, 0.55, 0.15, 0.95)
            },
        );
        draw_rectangle_lines(
            done_x,
            done_y,
            done_w,
            done_h,
            2.0,
            Color::new(1.0, 0.90, 0.45, 1.0),
        );
        dcx("BACK TO MENU", cx, done_y + 35.0, 22.0, BLACK, font);

        if done_hov && mouse_clicked {
            action = AchievementsAction::Close;
        }

        action
    }
}
