use crate::core::config::{ResolutionProfile, VIRTUAL_HEIGHT, VIRTUAL_WIDTH};
use crate::graphics::icons::draw_vector_close;
use macroquad::prelude::*;

#[derive(Debug, Clone)]
pub enum SettingsAction {
    None,
    ChangeResolution(ResolutionProfile),
    ToggleSound,
    ToggleHaptics,
    Close,
}

pub struct SettingsModal;

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

impl SettingsModal {
    pub fn draw(
        current_res: ResolutionProfile,
        sound_enabled: bool,
        haptics_enabled: bool,
        mouse_pos: Vec2,
        mouse_clicked: bool,
        font: Option<&Font>,
    ) -> SettingsAction {
        let mut action = SettingsAction::None;
        let cx = VIRTUAL_WIDTH * 0.5;

        // Dark ambient backdrop
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
            Color::new(0.35, 0.70, 1.0, 0.65),
        );

        // Header
        dcx("SETTINGS & DISPLAY", cx, card_y + 46.0, 32.0, WHITE, font);
        dcx(
            "Configure visual fidelity & hardware performance",
            cx,
            card_y + 74.0,
            16.0,
            Color::new(0.65, 0.82, 1.0, 0.85),
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
        draw_vector_close(
            close_btn_x + 16.0,
            close_btn_y + 16.0,
            10.0,
            10.0,
            close_col,
        );
        if (close_hov && mouse_clicked) || is_key_pressed(KeyCode::Escape) {
            action = SettingsAction::Close;
        }

        // ── Section 1: Resolution & Hardware Performance ─────────────────────────
        let sec_y = card_y + 115.0;
        dtx(
            "RENDER RESOLUTION",
            card_x + 24.0,
            sec_y,
            20.0,
            Color::new(0.40, 0.85, 1.0, 1.0),
            font,
        );
        dtx(
            "Internal buffer dynamically rescales to selected profile",
            card_x + 24.0,
            sec_y + 22.0,
            14.0,
            Color::new(0.60, 0.75, 0.90, 0.75),
            font,
        );

        let mut row_y = sec_y + 36.0;
        for profile in ResolutionProfile::ALL {
            let is_active = profile == current_res;
            let card_w = cw - 48.0;
            let card_h = 76.0;
            let hov = inside(mouse_pos, card_x + 24.0, row_y, card_w, card_h);

            let bg = if is_active {
                Color::new(0.12, 0.28, 0.52, 0.90)
            } else if hov {
                Color::new(0.10, 0.14, 0.26, 0.85)
            } else {
                Color::new(0.07, 0.09, 0.18, 0.75)
            };

            draw_rectangle(card_x + 24.0, row_y, card_w, card_h, bg);
            let border_col = if is_active {
                Color::new(0.40, 0.90, 1.0, 1.0)
            } else if hov {
                Color::new(0.50, 0.70, 0.95, 0.70)
            } else {
                Color::new(0.20, 0.30, 0.45, 0.50)
            };
            draw_rectangle_lines(
                card_x + 24.0,
                row_y,
                card_w,
                card_h,
                if is_active { 2.2 } else { 1.2 },
                border_col,
            );

            // Active Radio Pip
            let pip_cx = card_x + 48.0;
            let pip_cy = row_y + card_h * 0.5;
            draw_circle(pip_cx, pip_cy, 9.0, Color::new(0.15, 0.22, 0.36, 1.0));
            draw_circle_lines(pip_cx, pip_cy, 9.0, 1.5, border_col);
            if is_active {
                draw_circle(pip_cx, pip_cy, 5.0, Color::new(0.40, 0.95, 1.0, 1.0));
            }

            // Labels
            dtx(
                profile.label(),
                card_x + 72.0,
                row_y + 30.0,
                20.0,
                if is_active {
                    WHITE
                } else {
                    Color::new(0.85, 0.90, 0.98, 1.0)
                },
                font,
            );
            dtx(
                profile.recommendation(),
                card_x + 72.0,
                row_y + 54.0,
                14.0,
                if is_active {
                    Color::new(0.70, 0.92, 1.0, 0.95)
                } else {
                    Color::new(0.60, 0.70, 0.85, 0.70)
                },
                font,
            );

            // "ACTIVE" Badge
            if is_active {
                let badge_x = card_x + card_w - 75.0;
                let badge_y = row_y + 24.0;
                draw_rectangle(
                    badge_x,
                    badge_y,
                    75.0,
                    26.0,
                    Color::new(0.10, 0.65, 0.40, 0.95),
                );
                dcx("ACTIVE", badge_x + 37.5, badge_y + 19.0, 13.0, WHITE, font);
            }

            if hov && mouse_clicked && !is_active {
                action = SettingsAction::ChangeResolution(profile);
            }

            row_y += card_h + 12.0;
        }

        // ── Section 2: Audio & Feedback ──────────────────────────────────────────
        let audio_y = row_y + 20.0;
        dtx(
            "AUDIO & HAPTICS",
            card_x + 24.0,
            audio_y,
            20.0,
            Color::new(0.40, 0.85, 1.0, 1.0),
            font,
        );

        let toggle_w = (cw - 60.0) * 0.5;
        let toggle_h = 60.0;
        let t1_x = card_x + 24.0;
        let t2_x = t1_x + toggle_w + 12.0;
        let toggles_y = audio_y + 14.0;

        // Sound Toggle
        let hov_sound = inside(mouse_pos, t1_x, toggles_y, toggle_w, toggle_h);
        draw_rectangle(
            t1_x,
            toggles_y,
            toggle_w,
            toggle_h,
            if sound_enabled {
                Color::new(0.12, 0.32, 0.22, 0.85)
            } else {
                Color::new(0.12, 0.12, 0.18, 0.85)
            },
        );
        draw_rectangle_lines(
            t1_x,
            toggles_y,
            toggle_w,
            toggle_h,
            1.5,
            if sound_enabled {
                Color::new(0.35, 0.90, 0.55, 0.9)
            } else {
                Color::new(0.4, 0.4, 0.5, 0.6)
            },
        );
        dcx(
            if sound_enabled {
                "SOUND: ENABLED"
            } else {
                "SOUND: MUTED"
            },
            t1_x + toggle_w * 0.5,
            toggles_y + 36.0,
            17.0,
            WHITE,
            font,
        );
        if hov_sound && mouse_clicked {
            action = SettingsAction::ToggleSound;
        }

        // Haptics Toggle
        let hov_haptics = inside(mouse_pos, t2_x, toggles_y, toggle_w, toggle_h);
        draw_rectangle(
            t2_x,
            toggles_y,
            toggle_w,
            toggle_h,
            if haptics_enabled {
                Color::new(0.12, 0.32, 0.22, 0.85)
            } else {
                Color::new(0.12, 0.12, 0.18, 0.85)
            },
        );
        draw_rectangle_lines(
            t2_x,
            toggles_y,
            toggle_w,
            toggle_h,
            1.5,
            if haptics_enabled {
                Color::new(0.35, 0.90, 0.55, 0.9)
            } else {
                Color::new(0.4, 0.4, 0.5, 0.6)
            },
        );
        dcx(
            if haptics_enabled {
                "HAPTICS: ON"
            } else {
                "HAPTICS: OFF"
            },
            t2_x + toggle_w * 0.5,
            toggles_y + 36.0,
            17.0,
            WHITE,
            font,
        );
        if hov_haptics && mouse_clicked {
            action = SettingsAction::ToggleHaptics;
        }

        // Bottom Done / Close Button
        let done_w = 260.0;
        let done_h = 56.0;
        let done_x = cx - done_w * 0.5;
        let done_y = card_y + ch - 72.0;
        let done_hov = inside(mouse_pos, done_x, done_y, done_w, done_h);

        draw_rectangle(
            done_x,
            done_y,
            done_w,
            done_h,
            if done_hov {
                Color::new(0.20, 0.65, 0.95, 1.0)
            } else {
                Color::new(0.12, 0.45, 0.75, 0.95)
            },
        );
        draw_rectangle_lines(
            done_x,
            done_y,
            done_w,
            done_h,
            2.0,
            Color::new(0.60, 0.90, 1.0, 1.0),
        );
        dcx("SAVE & BACK", cx, done_y + 36.0, 22.0, WHITE, font);

        if done_hov && mouse_clicked {
            action = SettingsAction::Close;
        }

        action
    }
}
