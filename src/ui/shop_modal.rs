use macroquad::prelude::*;
use crate::core::config::{VIRTUAL_HEIGHT, VIRTUAL_WIDTH};
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
    draw_text_ex(text, x, y, TextParams { font, font_size: sz as u16, color: col, ..Default::default() });
}

fn dcx(text: &str, cx: f32, y: f32, sz: f32, col: Color, font: Option<&Font>) {
    let dim = measure_text(text, font, sz as u16, 1.0);
    draw_text_ex(text, cx - dim.width * 0.5, y, TextParams { font, font_size: sz as u16, color: col, ..Default::default() });
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
        let ch = VIRTUAL_HEIGHT - 100.0;
        let card_x = 16.0;
        let card_y = 50.0;
        draw_rectangle(card_x, card_y, cw, ch, Color::new(0.06, 0.05, 0.14, 0.97));
        draw_rectangle_lines(card_x, card_y, cw, ch, 2.0, Color::new(0.40, 0.80, 1.0, 0.65));

        // Title bar
        dcx("COSMIC STORE", cx, card_y + 48.0, 32.0, WHITE, font);
        let dust_str = format!("STARDUST  {}", stardust);
        dcx(&dust_str, cx, card_y + 80.0, 22.0, Color::new(0.42, 0.90, 1.0, 1.0), font);

        // Close button
        let close_x = card_x + cw - 55.0;
        let close_y = card_y + 12.0;
        let close_hov = inside(mouse_pos, close_x, close_y, 44.0, 44.0);
        draw_rectangle(
            close_x, close_y, 44.0, 44.0,
            if close_hov { Color::new(0.82, 0.22, 0.22, 0.90) } else { Color::new(0.32, 0.10, 0.10, 0.65) },
        );
        draw_rectangle_lines(close_x, close_y, 44.0, 44.0, 1.5, WHITE);
        dcx("X", close_x + 22.0, close_y + 30.0, 24.0, WHITE, font);
        if close_hov && mouse_clicked { action = ShopAction::Close; }

        // Status message
        if let Some(msg) = status_message {
            dcx(msg, cx, card_y + 108.0, 20.0, Color::new(0.30, 1.0, 0.50, 1.0), font);
        }

        let mut cur_y = card_y + 128.0;

        // ── IAP items ────────────────────────────────────────────────────────
        dcx("IN-APP PURCHASES", cx, cur_y + 22.0, 20.0, Color::new(0.95, 0.82, 0.30, 1.0), font);
        cur_y += 36.0;

        for item in &catalog.iap_items {
            let is_owned = item.id.contains("removeads") && ads_removed;
            let ih = 78.0;
            let iw = cw - 32.0;
            let ix = card_x + 16.0;

            draw_rectangle(ix, cur_y, iw, ih, Color::new(0.11, 0.09, 0.20, 0.82));
            draw_rectangle_lines(ix, cur_y, iw, ih, 1.0, Color::new(0.30, 0.40, 0.60, 0.45));

            dtx(&item.title, ix + 14.0, cur_y + 30.0, 22.0, WHITE, font);
            dtx(&item.description, ix + 14.0, cur_y + 58.0, 16.0, Color::new(0.72, 0.76, 0.86, 0.75), font);

            let bw2 = 120.0;
            let bh = 46.0;
            let bx2 = ix + iw - bw2 - 12.0;
            let by2 = cur_y + 16.0;

            if is_owned {
                draw_rectangle(bx2, by2, bw2, bh, Color::new(0.18, 0.40, 0.24, 0.65));
                dcx("ACTIVE", bx2 + bw2 * 0.5, by2 + 30.0, 18.0, Color::new(0.5, 0.95, 0.62, 1.0), font);
            } else {
                let hov = inside(mouse_pos, bx2, by2, bw2, bh);
                draw_rectangle(bx2, by2, bw2, bh,
                    if hov { Color::new(0.28, 0.80, 0.44, 1.0) } else { Color::new(0.18, 0.62, 0.32, 1.0) });
                draw_rectangle_lines(bx2, by2, bw2, bh, 1.2, WHITE);
                dcx(&item.price_display, bx2 + bw2 * 0.5, by2 + 30.0, 20.0, WHITE, font);
                if hov && mouse_clicked { action = ShopAction::BuyItem(item.id.clone()); }
            }
            cur_y += ih + 10.0;
        }

        // ── Cosmetic skins ────────────────────────────────────────────────────
        cur_y += 10.0;
        dcx("COSMETIC THEMES", cx, cur_y + 22.0, 20.0, Color::new(0.42, 0.90, 1.0, 1.0), font);
        cur_y += 36.0;

        for item in &catalog.skin_items {
            let is_unlocked = unlocked_skins.contains(&item.id);
            let is_equipped = equipped_skin == item.id;
            let ih = 72.0;
            let iw = cw - 32.0;
            let ix = card_x + 16.0;

            draw_rectangle(ix, cur_y, iw, ih, Color::new(0.11, 0.09, 0.20, 0.82));
            draw_rectangle_lines(ix, cur_y, iw, ih, 1.0, Color::new(0.30, 0.40, 0.60, 0.45));

            dtx(&item.title, ix + 14.0, cur_y + 28.0, 21.0, WHITE, font);
            dtx(&item.description, ix + 14.0, cur_y + 54.0, 16.0, Color::new(0.72, 0.76, 0.86, 0.75), font);

            let bw2 = 120.0;
            let bh = 44.0;
            let bx2 = ix + iw - bw2 - 12.0;
            let by2 = cur_y + 14.0;

            if is_equipped {
                draw_rectangle(bx2, by2, bw2, bh, Color::new(0.20, 0.50, 0.72, 0.75));
                dcx("EQUIPPED", bx2 + bw2 * 0.5, by2 + 28.0, 18.0, WHITE, font);
            } else if is_unlocked {
                let hov = inside(mouse_pos, bx2, by2, bw2, bh);
                draw_rectangle(bx2, by2, bw2, bh,
                    if hov { Color::new(0.36, 0.58, 0.90, 1.0) } else { Color::new(0.26, 0.46, 0.74, 1.0) });
                draw_rectangle_lines(bx2, by2, bw2, bh, 1.2, WHITE);
                dcx("EQUIP", bx2 + bw2 * 0.5, by2 + 28.0, 18.0, WHITE, font);
                if hov && mouse_clicked { action = ShopAction::EquipSkin(item.id.clone()); }
            } else {
                let can = stardust >= item.stardust_price;
                let hov = inside(mouse_pos, bx2, by2, bw2, bh);
                draw_rectangle(bx2, by2, bw2, bh,
                    if can && hov { Color::new(0.88, 0.62, 0.22, 1.0) }
                    else if can   { Color::new(0.76, 0.50, 0.15, 1.0) }
                    else          { Color::new(0.28, 0.28, 0.34, 0.65) });
                draw_rectangle_lines(bx2, by2, bw2, bh, 1.2, WHITE);
                dcx(&item.price_display, bx2 + bw2 * 0.5, by2 + 28.0, 18.0, WHITE, font);
                if can && hov && mouse_clicked { action = ShopAction::BuyItem(item.id.clone()); }
            }
            cur_y += ih + 8.0;
        }

        action
    }
}
