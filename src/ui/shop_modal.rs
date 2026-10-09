use macroquad::prelude::*;
use crate::core::config::{VIRTUAL_HEIGHT, VIRTUAL_WIDTH};
use crate::graphics::icons::{draw_vector_close, draw_vector_gem};
use crate::monetization::economy::EconomyCatalog;

#[derive(Debug, Clone)]
pub enum ShopAction {
    None,
    BuyItem(String),
    EquipSkin(String),
    Close,
}

pub struct ShopModal;

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

impl ShopModal {
    #[allow(clippy::too_many_arguments)]
    pub fn draw(
        catalog: &EconomyCatalog,
        stardust: u64,
        ads_removed: bool,
        equipped_skin: &str,
        unlocked_skins: &[String],
        status_message: Option<&str>,
        mouse_pos: Vec2,
        mouse_clicked: bool,
        font: Option<&Font>,
    ) -> ShopAction {
        let mut action = ShopAction::None;
        let cx = VIRTUAL_WIDTH * 0.5;

        // Dim backdrop
        draw_rectangle(0.0, 0.0, VIRTUAL_WIDTH, VIRTUAL_HEIGHT, Color::new(0.0, 0.0, 0.0, 0.88));

        // Full-screen card
        let cw = VIRTUAL_WIDTH - 32.0;
        let ch = VIRTUAL_HEIGHT - 80.0;
        let card_x = 16.0;
        let card_y = 40.0;
        draw_rectangle(card_x, card_y, cw, ch, Color::new(0.06, 0.05, 0.14, 0.98));
        draw_rectangle_lines(card_x, card_y, cw, ch, 2.0, Color::new(0.40, 0.80, 1.0, 0.65));

        // Title bar
        dcx("COSMIC STORE", cx, card_y + 44.0, 32.0, WHITE, font);

        // Stardust balance badge
        let dust_card_w = 200.0;
        let dust_card_x = cx - dust_card_w * 0.5;
        let dust_card_y = card_y + 58.0;
        draw_rectangle(dust_card_x, dust_card_y, dust_card_w, 36.0, Color::new(0.10, 0.10, 0.22, 0.85));
        draw_rectangle_lines(dust_card_x, dust_card_y, dust_card_w, 36.0, 1.0, Color::new(0.35, 0.65, 0.95, 0.50));
        draw_vector_gem(dust_card_x + 24.0, dust_card_y + 18.0, 16.0, Color::new(0.35, 0.85, 1.0, 1.0));
        dtx(&format!("STARDUST: {}", stardust), dust_card_x + 42.0, dust_card_y + 25.0, 18.0, Color::new(0.85, 0.95, 1.0, 1.0), font);

        // Close button (expanded hit box for mobile thumbs)
        let close_x = card_x + cw - 52.0;
        let close_y = card_y + 14.0;
        let close_hov = inside(mouse_pos, close_x - 8.0, close_y - 8.0, 54.0, 54.0);
        draw_rectangle(
            close_x, close_y, 38.0, 38.0,
            if close_hov { Color::new(0.82, 0.22, 0.22, 0.90) } else { Color::new(0.25, 0.12, 0.16, 0.70) },
        );
        draw_rectangle_lines(close_x, close_y, 38.0, 38.0, 1.5, WHITE);
        draw_vector_close(close_x + 19.0, close_y + 19.0, 16.0, 2.0, WHITE);
        if (close_hov && mouse_clicked) || is_key_pressed(KeyCode::Escape) { action = ShopAction::Close; }

        // Status message
        if let Some(msg) = status_message {
            dcx(msg, cx, card_y + 116.0, 18.0, Color::new(0.30, 1.0, 0.50, 1.0), font);
        }

        let mut cur_y = card_y + 130.0;

        // ── In-App Purchases & Stardust Packs ────────────────────────────────
        dcx("IN-APP PURCHASES & PACKS", cx, cur_y + 18.0, 20.0, Color::new(0.95, 0.82, 0.30, 1.0), font);
        cur_y += 32.0;

        for item in &catalog.iap_items {
            let is_owned = item.id.contains("removeads") && ads_removed;
            let ih = 68.0;
            let iw = cw - 32.0;
            let ix = card_x + 16.0;

            draw_rectangle(ix, cur_y, iw, ih, Color::new(0.11, 0.09, 0.20, 0.85));
            draw_rectangle_lines(ix, cur_y, iw, ih, 1.0, Color::new(0.30, 0.40, 0.60, 0.45));

            dtx(&item.title, ix + 14.0, cur_y + 26.0, 20.0, WHITE, font);
            dtx(&item.description, ix + 14.0, cur_y + 50.0, 15.0, Color::new(0.82, 0.88, 0.98, 0.95), font);

            let bw2 = 120.0;
            let bh = 42.0;
            let bx2 = ix + iw - bw2 - 12.0;
            let by2 = cur_y + 13.0;

            if is_owned {
                draw_rectangle(bx2, by2, bw2, bh, Color::new(0.18, 0.18, 0.24, 0.50));
                dcx("OWNED", bx2 + bw2 * 0.5, by2 + 26.0, 17.0, Color::new(0.50, 0.80, 0.50, 0.85), font);
            } else {
                let hov2 = inside(mouse_pos, bx2, by2, bw2, bh);
                draw_rectangle(bx2, by2, bw2, bh,
                    if hov2 { Color::new(0.28, 0.82, 0.45, 1.0) } else { Color::new(0.18, 0.64, 0.34, 1.0) });
                draw_rectangle_lines(bx2, by2, bw2, bh, 1.5, WHITE);
                dcx(&item.price_display, bx2 + bw2 * 0.5, by2 + 26.0, 17.0, WHITE, font);
                if hov2 && mouse_clicked { action = ShopAction::BuyItem(item.id.clone()); }
            }

            cur_y += ih + 10.0;
        }

        // ── Cosmic Skins ─────────────────────────────────────────────────────
        cur_y += 12.0;
        dcx("COSMIC SKINS", cx, cur_y + 18.0, 20.0, Color::new(0.85, 0.45, 1.0, 1.0), font);
        cur_y += 32.0;

        for skin in &catalog.skin_items {
            let is_unlocked = unlocked_skins.contains(&skin.id);
            let is_equipped = equipped_skin == skin.id;
            let ih = 64.0;
            let iw = cw - 32.0;
            let ix = card_x + 16.0;

            draw_rectangle(ix, cur_y, iw, ih, Color::new(0.12, 0.08, 0.20, 0.85));
            draw_rectangle_lines(ix, cur_y, iw, ih, 1.0, Color::new(0.50, 0.30, 0.70, 0.45));

            dtx(&skin.title, ix + 14.0, cur_y + 26.0, 20.0, WHITE, font);
            dtx(&skin.description, ix + 14.0, cur_y + 48.0, 15.0, Color::new(0.82, 0.88, 0.98, 0.95), font);

            let bw2 = 110.0;
            let bh = 40.0;
            let bx2 = ix + iw - bw2 - 12.0;
            let by2 = cur_y + 12.0;

            if is_equipped {
                draw_rectangle(bx2, by2, bw2, bh, Color::new(0.20, 0.50, 0.30, 0.75));
                dcx("EQUIPPED", bx2 + bw2 * 0.5, by2 + 26.0, 16.0, Color::new(0.60, 1.0, 0.60, 1.0), font);
            } else if is_unlocked {
                let hov2 = inside(mouse_pos, bx2, by2, bw2, bh);
                draw_rectangle(bx2, by2, bw2, bh,
                    if hov2 { Color::new(0.45, 0.30, 0.75, 1.0) } else { Color::new(0.32, 0.20, 0.58, 1.0) });
                draw_rectangle_lines(bx2, by2, bw2, bh, 1.5, WHITE);
                dcx("EQUIP", bx2 + bw2 * 0.5, by2 + 26.0, 17.0, WHITE, font);
                if hov2 && mouse_clicked { action = ShopAction::EquipSkin(skin.id.clone()); }
            } else {
                let can_afford = stardust >= skin.stardust_price;
                let hov2 = inside(mouse_pos, bx2, by2, bw2, bh) && can_afford;
                let bg_col = if !can_afford {
                    Color::new(0.18, 0.18, 0.24, 0.50)
                } else if hov2 {
                    Color::new(0.75, 0.40, 0.90, 1.0)
                } else {
                    Color::new(0.55, 0.25, 0.72, 1.0)
                };
                draw_rectangle(bx2, by2, bw2, bh, bg_col);
                if can_afford { draw_rectangle_lines(bx2, by2, bw2, bh, 1.5, WHITE); }
                let pr_str = format!("{} DUST", skin.stardust_price);
                dcx(&pr_str, bx2 + bw2 * 0.5, by2 + 26.0, 15.0, if can_afford { WHITE } else { Color::new(0.5,0.5,0.6,0.8) }, font);
                if hov2 && mouse_clicked { action = ShopAction::BuyItem(skin.id.clone()); }
            }

            cur_y += ih + 8.0;
        }

        action
    }
}
