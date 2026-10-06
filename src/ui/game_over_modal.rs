use crate::core::config::{VIRTUAL_HEIGHT, VIRTUAL_WIDTH};
use crate::graphics::icons::draw_vector_gem;
use macroquad::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameOverAction {
    None,
    WatchAdRevive,
    WatchAdDoubleStardust,
    Restart,
    OpenShop,
    SubmitLeaderboard,
}

pub struct GameOverModal;

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

impl GameOverModal {
    #[allow(clippy::too_many_arguments)]
    pub fn draw(
        score: u64,
        high_score: u64,
        stardust_earned: u64,
        revive_available: bool,
        stardust_doubled: bool,
        public_name: &str,
        leaderboard_status: &str,
        mouse_pos: Vec2,
        mouse_clicked: bool,
        font: Option<&Font>,
    ) -> GameOverAction {
        let mut action = GameOverAction::None;
        let cx = VIRTUAL_WIDTH * 0.5;

        // Dim backdrop
        draw_rectangle(
            0.0,
            0.0,
            VIRTUAL_WIDTH,
            VIRTUAL_HEIGHT,
            Color::new(0.0, 0.0, 0.0, 0.82),
        );

        // Card
        let cw = VIRTUAL_WIDTH - 64.0;
        let ch = 760.0;
        let cx2 = 32.0;
        let cy = (VIRTUAL_HEIGHT - ch) * 0.5;
        draw_rectangle(cx2, cy, cw, ch, Color::new(0.07, 0.06, 0.16, 0.98));
        draw_rectangle_lines(cx2, cy, cw, ch, 2.0, Color::new(0.85, 0.35, 1.0, 0.70));

        // Title
        dcx(
            "CONTAINMENT OVERFLOW",
            cx,
            cy + 54.0,
            32.0,
            Color::new(1.0, 0.35, 0.45, 1.0),
            font,
        );

        // Stats plaque
        let py = cy + 78.0;
        draw_rectangle(
            cx2 + 16.0,
            py,
            cw - 32.0,
            154.0,
            Color::new(0.12, 0.10, 0.22, 0.88),
        );
        draw_rectangle_lines(
            cx2 + 16.0,
            py,
            cw - 32.0,
            154.0,
            1.0,
            Color::new(0.40, 0.35, 0.60, 0.45),
        );

        dtx(
            "FINAL SCORE",
            cx2 + 36.0,
            py + 36.0,
            18.0,
            Color::new(0.70, 0.75, 0.90, 0.85),
            font,
        );
        let sc = score.to_string();
        dtx(&sc, cx2 + 36.0, py + 86.0, 50.0, WHITE, font);

        if score >= high_score && score > 0 {
            dtx(
                "NEW HIGH SCORE!",
                cx2 + 36.0,
                py + 118.0,
                18.0,
                Color::new(1.0, 0.88, 0.20, 1.0),
                font,
            );
        } else {
            let ht = format!("BEST {}", high_score);
            dtx(
                &ht,
                cx2 + 36.0,
                py + 118.0,
                18.0,
                Color::new(0.70, 0.70, 0.82, 0.75),
                font,
            );
        }

        // Stardust badge in plaque
        draw_vector_gem(
            cx2 + 48.0,
            py + 140.0,
            18.0,
            Color::new(0.35, 0.85, 1.0, 1.0),
        );
        let dust_lbl = if stardust_doubled {
            format!("+{} STARDUST (DOUBLED!)", stardust_earned)
        } else {
            format!("+{} STARDUST", stardust_earned)
        };
        dtx(
            &dust_lbl,
            cx2 + 66.0,
            py + 146.0,
            20.0,
            Color::new(0.42, 0.90, 1.0, 1.0),
            font,
        );

        // Public leaderboard name prompt shown after every finished run.
        dtx(
            "PUBLIC LEADERBOARD NAME",
            cx2 + 22.0,
            py + 184.0,
            17.0,
            Color::new(0.60, 0.88, 1.0, 1.0),
            font,
        );
        draw_rectangle(
            cx2 + 18.0,
            py + 193.0,
            cw - 36.0,
            48.0,
            Color::new(0.08, 0.08, 0.16, 1.0),
        );
        draw_rectangle_lines(
            cx2 + 18.0,
            py + 193.0,
            cw - 36.0,
            48.0,
            1.3,
            Color::new(0.35, 0.70, 0.95, 0.85),
        );
        let shown_name = if public_name.is_empty() {
            "Type a name (3-20 characters)"
        } else {
            public_name
        };
        dtx(
            shown_name,
            cx2 + 30.0,
            py + 224.0,
            19.0,
            if public_name.is_empty() {
                Color::new(0.58, 0.60, 0.68, 0.85)
            } else {
                WHITE
            },
            font,
        );
        if !leaderboard_status.is_empty() {
            dtx(
                leaderboard_status,
                cx2 + 22.0,
                py + 266.0,
                15.0,
                Color::new(1.0, 0.68, 0.42, 1.0),
                font,
            );
        }
        let submit_y = cy + 350.0;
        let submit_x = cx2 + 18.0;
        let submit_hov = inside(mouse_pos, submit_x, submit_y, cw - 36.0, 52.0);
        draw_rectangle(
            submit_x,
            submit_y,
            cw - 36.0,
            52.0,
            if submit_hov {
                Color::new(0.20, 0.65, 0.86, 1.0)
            } else {
                Color::new(0.13, 0.49, 0.72, 1.0)
            },
        );
        draw_rectangle_lines(submit_x, submit_y, cw - 36.0, 52.0, 1.5, WHITE);
        dcx(
            "SUBMIT SCORE & NAME",
            cx,
            submit_y + 34.0,
            20.0,
            WHITE,
            font,
        );
        if submit_hov && mouse_clicked && !public_name.trim().is_empty() {
            action = GameOverAction::SubmitLeaderboard;
        }

        // Buttons
        let bw = cw - 36.0;
        let bx = cx2 + 18.0;
        let btn_h = 62.0;
        let mut by = cy + 412.0;

        // Button 1: Revive via ad
        if revive_available {
            let hov = inside(mouse_pos, bx, by, bw, btn_h);
            draw_rectangle(
                bx,
                by,
                bw,
                btn_h,
                if hov {
                    Color::new(0.30, 0.78, 0.46, 1.0)
                } else {
                    Color::new(0.20, 0.62, 0.36, 1.0)
                },
            );
            draw_rectangle_lines(bx, by, bw, btn_h, 1.8, WHITE);
            dcx(
                "WATCH AD: REWIND & REVIVE",
                cx,
                by + 40.0,
                22.0,
                WHITE,
                font,
            );
            if hov && mouse_clicked {
                action = GameOverAction::WatchAdRevive;
            }
        } else {
            draw_rectangle(bx, by, bw, btn_h, Color::new(0.18, 0.18, 0.24, 0.55));
            dcx(
                "REVIVE USED",
                cx,
                by + 40.0,
                20.0,
                Color::new(0.50, 0.50, 0.58, 1.0),
                font,
            );
        }
        by += btn_h + 14.0;

        // Button 2: Double stardust via ad
        if !stardust_doubled && stardust_earned > 0 {
            let hov = inside(mouse_pos, bx, by, bw, btn_h);
            draw_rectangle(
                bx,
                by,
                bw,
                btn_h,
                if hov {
                    Color::new(0.88, 0.58, 0.18, 1.0)
                } else {
                    Color::new(0.72, 0.46, 0.10, 1.0)
                },
            );
            draw_rectangle_lines(bx, by, bw, btn_h, 1.8, WHITE);
            dcx("WATCH AD: 2X STARDUST", cx, by + 40.0, 22.0, WHITE, font);
            if hov && mouse_clicked {
                action = GameOverAction::WatchAdDoubleStardust;
            }
        } else {
            draw_rectangle(bx, by, bw, btn_h, Color::new(0.18, 0.18, 0.24, 0.40));
            dcx(
                "STARDUST CLAIMED",
                cx,
                by + 40.0,
                20.0,
                Color::new(0.50, 0.50, 0.58, 0.75),
                font,
            );
        }
        by += btn_h + 14.0;

        // Button 3: Play again
        let hov3 = inside(mouse_pos, bx, by, bw, btn_h);
        draw_rectangle(
            bx,
            by,
            bw,
            btn_h,
            if hov3 {
                Color::new(0.32, 0.48, 0.88, 1.0)
            } else {
                Color::new(0.22, 0.36, 0.72, 1.0)
            },
        );
        draw_rectangle_lines(bx, by, bw, btn_h, 1.8, WHITE);
        dcx("PLAY AGAIN", cx, by + 40.0, 24.0, WHITE, font);
        if hov3 && mouse_clicked {
            action = GameOverAction::Restart;
        }
        by += btn_h + 14.0;

        // Button 4: Shop
        let hov4 = inside(mouse_pos, bx, by, bw, btn_h);
        draw_rectangle(
            bx,
            by,
            bw,
            btn_h,
            if hov4 {
                Color::new(0.62, 0.28, 0.80, 1.0)
            } else {
                Color::new(0.46, 0.18, 0.62, 1.0)
            },
        );
        draw_rectangle_lines(bx, by, bw, btn_h, 1.8, WHITE);
        dcx("COSMIC SHOP", cx, by + 40.0, 24.0, WHITE, font);
        if hov4 && mouse_clicked {
            action = GameOverAction::OpenShop;
        }

        action
    }
}
