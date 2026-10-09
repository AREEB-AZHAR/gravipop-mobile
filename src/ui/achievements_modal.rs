use crate::core::achievements::{AchievementCategory, ALL_ACHIEVEMENTS};
use crate::core::config::{VIRTUAL_HEIGHT, VIRTUAL_WIDTH};
use crate::graphics::icons::{draw_vector_close, draw_vector_star};
use macroquad::prelude::*;

#[derive(Debug, Clone)]
pub enum AchievementsAction {
    None,
    Close,
}

pub struct AchievementsModal;

fn dtx(text: &str, x: f32, y: f32, sz: f32, col: Color, font: Option<&Font>) {
    let raster_size = (sz * 2.0).round().max(16.0) as u16;
    let font_scale = 0.5;
    let px = x.round();
    let py = y.round();
    draw_text_ex(
        text,
        px + 0.65,
        py,
        TextParams {
            font,
            font_size: raster_size,
            font_scale,
            color: col,
            ..Default::default()
        },
    );
    draw_text_ex(
        text,
        px,
        py,
        TextParams {
            font,
            font_size: raster_size,
            font_scale,
            color: col,
            ..Default::default()
        },
    );
}

fn dcx(text: &str, cx: f32, y: f32, sz: f32, col: Color, font: Option<&Font>) {
    let raster_size = (sz * 2.0).round().max(16.0) as u16;
    let font_scale = 0.5;
    let dim = measure_text(text, font, raster_size, font_scale);
    let px = (cx - dim.width * 0.5).round();
    let py = y.round();
    draw_text_ex(
        text,
        px + 0.65,
        py,
        TextParams {
            font,
            font_size: raster_size,
            font_scale,
            color: col,
            ..Default::default()
        },
    );
    draw_text_ex(
        text,
        px,
        py,
        TextParams {
            font,
            font_size: raster_size,
            font_scale,
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
        page: &mut usize,
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
        let ch = 1040.0;
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
        dcx("COSMIC ACHIEVEMENTS", cx, card_y + 44.0, 32.0, WHITE, font);

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
            card_y + 72.0,
            16.0,
            Color::new(1.0, 0.85, 0.40, 0.95),
            font,
        );

        // Close Button (top-right X)
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

        // ── Pagination Calculation ──
        const ITEMS_PER_PAGE: usize = 7;
        let total_pages = (ALL_ACHIEVEMENTS.len() + ITEMS_PER_PAGE - 1) / ITEMS_PER_PAGE;
        if *page >= total_pages {
            *page = total_pages.saturating_sub(1);
        }

        let start_idx = *page * ITEMS_PER_PAGE;
        let end_idx = (start_idx + ITEMS_PER_PAGE).min(ALL_ACHIEVEMENTS.len());

        // Paged List of Achievements
        let mut row_y = card_y + 96.0;
        let item_w = cw - 32.0;
        let item_h = 86.0;
        let item_x = card_x + 16.0;

        for ach in &ALL_ACHIEVEMENTS[start_idx..end_idx] {
            let is_unlocked = unlocked_ids.iter().any(|id| id == ach.id);
            let bg = if is_unlocked {
                Color::new(0.10, 0.16, 0.28, 0.92)
            } else {
                Color::new(0.06, 0.08, 0.15, 0.85)
            };

            draw_rectangle(item_x, row_y, item_w, item_h, bg);

            let border_col = if is_unlocked {
                Color::new(0.95, 0.80, 0.30, 0.85)
            } else {
                Color::new(0.24, 0.30, 0.45, 0.50)
            };
            draw_rectangle_lines(item_x, row_y, item_w, item_h, 1.4, border_col);

            // Icon Pip
            let icon_cx = item_x + 28.0;
            let icon_cy = row_y + item_h * 0.5;
            if is_unlocked {
                draw_circle(icon_cx, icon_cy, 18.0, Color::new(0.22, 0.35, 0.60, 0.90));
                draw_vector_star(icon_cx, icon_cy, 10.0, Color::new(1.0, 0.84, 0.25, 1.0));
            } else {
                draw_circle(icon_cx, icon_cy, 16.0, Color::new(0.12, 0.14, 0.22, 0.90));
                draw_circle_lines(icon_cx, icon_cy, 16.0, 1.2, Color::new(0.30, 0.35, 0.50, 0.50));
            }

            // Title & Description (High Contrast & Clear Sizing)
            let text_x = item_x + 58.0;
            dtx(
                ach.title,
                text_x,
                row_y + 32.0,
                20.0,
                if is_unlocked {
                    WHITE
                } else {
                    Color::new(0.85, 0.88, 0.96, 0.95)
                },
                font,
            );
            dtx(
                ach.description,
                text_x,
                row_y + 60.0,
                15.0,
                if is_unlocked {
                    Color::new(0.75, 0.90, 1.0, 0.95)
                } else {
                    Color::new(0.60, 0.70, 0.85, 0.88)
                },
                font,
            );

            // Status Badge
            let badge_w = 95.0;
            let badge_h = 32.0;
            let badge_x = item_x + item_w - badge_w - 14.0;
            let badge_y = row_y + (item_h - badge_h) * 0.5;

            if is_unlocked {
                draw_rectangle(
                    badge_x,
                    badge_y,
                    badge_w,
                    badge_h,
                    Color::new(0.12, 0.60, 0.32, 0.95),
                );
                draw_rectangle_lines(badge_x, badge_y, badge_w, badge_h, 1.2, Color::new(0.40, 1.0, 0.60, 0.80));
                dcx(
                    "CLAIMED",
                    badge_x + badge_w * 0.5,
                    badge_y + 22.0,
                    14.0,
                    WHITE,
                    font,
                );
            } else {
                draw_rectangle(
                    badge_x,
                    badge_y,
                    badge_w,
                    badge_h,
                    Color::new(0.12, 0.14, 0.22, 0.88),
                );
                draw_rectangle_lines(badge_x, badge_y, badge_w, badge_h, 1.0, Color::new(0.30, 0.35, 0.48, 0.60));
                if ach.category == AchievementCategory::Score {
                    let pct =
                        ((current_score as f32 / ach.target_value as f32).min(1.0) * 100.0) as u32;
                    dcx(
                        &format!("{}%", pct),
                        badge_x + badge_w * 0.5,
                        badge_y + 22.0,
                        14.0,
                        Color::new(0.70, 0.85, 1.0, 0.90),
                        font,
                    );
                } else {
                    dcx(
                        "LOCKED",
                        badge_x + badge_w * 0.5,
                        badge_y + 22.0,
                        14.0,
                        Color::new(0.60, 0.65, 0.78, 0.80),
                        font,
                    );
                }
            }

            row_y += item_h + 12.0;
        }

        // ── Pagination Controls ──
        let nav_y = card_y + 830.0;
        let nav_btn_w = 130.0;
        let nav_btn_h = 44.0;

        // Previous Page Button
        let prev_x = card_x + 20.0;
        let can_prev = *page > 0;
        let prev_hov = can_prev && inside(mouse_pos, prev_x, nav_y, nav_btn_w, nav_btn_h);
        draw_rectangle(
            prev_x,
            nav_y,
            nav_btn_w,
            nav_btn_h,
            if !can_prev {
                Color::new(0.10, 0.11, 0.18, 0.50)
            } else if prev_hov {
                Color::new(0.25, 0.45, 0.80, 0.95)
            } else {
                Color::new(0.15, 0.25, 0.48, 0.85)
            },
        );
        draw_rectangle_lines(
            prev_x,
            nav_y,
            nav_btn_w,
            nav_btn_h,
            1.2,
            if can_prev { Color::new(0.40, 0.70, 1.0, 0.80) } else { Color::new(0.20, 0.25, 0.35, 0.40) },
        );
        dcx(
            "< PREV",
            prev_x + nav_btn_w * 0.5,
            nav_y + 28.0,
            16.0,
            if can_prev { WHITE } else { Color::new(0.40, 0.45, 0.55, 0.50) },
            font,
        );
        if prev_hov && mouse_clicked {
            *page = page.saturating_sub(1);
        }

        // Page Indicator
        let page_lbl = format!("PAGE {} OF {}", *page + 1, total_pages);
        dcx(&page_lbl, cx, nav_y + 28.0, 17.0, Color::new(0.85, 0.92, 1.0, 0.95), font);

        // Next Page Button
        let next_x = card_x + cw - nav_btn_w - 20.0;
        let can_next = *page + 1 < total_pages;
        let next_hov = can_next && inside(mouse_pos, next_x, nav_y, nav_btn_w, nav_btn_h);
        draw_rectangle(
            next_x,
            nav_y,
            nav_btn_w,
            nav_btn_h,
            if !can_next {
                Color::new(0.10, 0.11, 0.18, 0.50)
            } else if next_hov {
                Color::new(0.25, 0.45, 0.80, 0.95)
            } else {
                Color::new(0.15, 0.25, 0.48, 0.85)
            },
        );
        draw_rectangle_lines(
            next_x,
            nav_y,
            nav_btn_w,
            nav_btn_h,
            1.2,
            if can_next { Color::new(0.40, 0.70, 1.0, 0.80) } else { Color::new(0.20, 0.25, 0.35, 0.40) },
        );
        dcx(
            "NEXT >",
            next_x + nav_btn_w * 0.5,
            nav_y + 28.0,
            16.0,
            if can_next { WHITE } else { Color::new(0.40, 0.45, 0.55, 0.50) },
            font,
        );
        if next_hov && mouse_clicked {
            *page = (*page + 1).min(total_pages - 1);
        }

        // Close / Back button
        let done_w = 280.0;
        let done_h = 56.0;
        let done_x = cx - done_w * 0.5;
        let done_y = card_y + 900.0;
        let done_hov = inside(mouse_pos, done_x, done_y, done_w, done_h);

        draw_rectangle(
            done_x,
            done_y,
            done_w,
            done_h,
            if done_hov {
                Color::new(0.95, 0.75, 0.25, 1.0)
            } else {
                Color::new(0.80, 0.60, 0.18, 0.95)
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
        dcx("BACK TO MENU", cx, done_y + 36.0, 22.0, BLACK, font);

        if done_hov && mouse_clicked {
            action = AchievementsAction::Close;
        }

        action
    }
}
