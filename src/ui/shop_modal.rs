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

impl ShopModal {
    pub fn draw(
        catalog: &EconomyCatalog,
        stardust: u64,
        ads_removed: bool,
        equipped_skin: &str,
        unlocked_skins: &[String],
        status_message: Option<&str>,
        mouse_pos: Vec2,
        mouse_clicked: bool,
    ) -> ShopAction {
        let mut action = ShopAction::None;

        // Dark dim backdrop
        draw_rectangle(0.0, 0.0, VIRTUAL_WIDTH, VIRTUAL_HEIGHT, Color::new(0.0, 0.0, 0.0, 0.85));

        // Center Glassmorphic Card
        let card_w = VIRTUAL_WIDTH - 40.0;
        let card_h = VIRTUAL_HEIGHT - 120.0;
        let card_x = 20.0;
        let card_y = 60.0;

        draw_rectangle(card_x, card_y, card_w, card_h, Color::new(0.06, 0.05, 0.14, 0.96));
        draw_rectangle_lines(card_x, card_y, card_w, card_h, 2.0, Color::new(0.4, 0.8, 1.0, 0.65));

        // Title Bar
        draw_text("COSMIC STORE", card_x + 30.0, card_y + 45.0, 28.0, WHITE);
        let dust_label = format!("✦ {}", stardust);
        draw_text(&dust_label, card_x + card_w - 180.0, card_y + 45.0, 24.0, Color::new(0.4, 0.9, 1.0, 1.0));

        // Close Button (Top right X)
        let close_btn_size = 40.0;
        let close_btn_x = card_x + card_w - 50.0;
        let close_btn_y = card_y + 15.0;
        let close_hover = is_inside(mouse_pos, close_btn_x, close_btn_y, close_btn_size, close_btn_size);
        draw_rectangle(close_btn_x, close_btn_y, close_btn_size, close_btn_size, if close_hover { Color::new(0.8, 0.2, 0.2, 0.8) } else { Color::new(0.3, 0.1, 0.1, 0.6) });
        draw_rectangle_lines(close_btn_x, close_btn_y, close_btn_size, close_btn_size, 1.0, WHITE);
        draw_text("X", close_btn_x + 14.0, close_btn_y + 28.0, 22.0, WHITE);
        if close_hover && mouse_clicked {
            action = ShopAction::Close;
        }

        // Optional status message
        if let Some(msg) = status_message {
            draw_text(msg, card_x + 30.0, card_y + 75.0, 18.0, Color::new(0.3, 1.0, 0.5, 1.0));
        }

        // SECTION 1: IN-APP PURCHASES
        let mut cur_y = card_y + 105.0;
        draw_text("IN-APP PURCHASES", card_x + 30.0, cur_y, 18.0, Color::new(0.9, 0.8, 0.3, 1.0));
        cur_y += 18.0;

        for item in &catalog.iap_items {
            let is_owned = item.id.contains("removeads") && ads_removed;
            let item_h = 70.0;
            let item_w = card_w - 40.0;
            let item_x = card_x + 20.0;

            draw_rectangle(item_x, cur_y, item_w, item_h, Color::new(0.11, 0.09, 0.20, 0.8));
            draw_rectangle_lines(item_x, cur_y, item_w, item_h, 1.0, Color::new(0.3, 0.4, 0.6, 0.4));

            draw_text(&item.title, item_x + 15.0, cur_y + 28.0, 20.0, WHITE);
            draw_text(&item.description, item_x + 15.0, cur_y + 52.0, 13.0, Color::new(0.7, 0.75, 0.85, 0.7));

            // Price / Buy Button
            let btn_w = 110.0;
            let btn_h = 42.0;
            let btn_x = item_x + item_w - btn_w - 15.0;
            let btn_y = cur_y + 14.0;

            if is_owned {
                draw_rectangle(btn_x, btn_y, btn_w, btn_h, Color::new(0.2, 0.4, 0.25, 0.6));
                draw_text("ACTIVE", btn_x + 22.0, btn_y + 27.0, 16.0, Color::new(0.5, 0.9, 0.6, 1.0));
            } else {
                let btn_hover = is_inside(mouse_pos, btn_x, btn_y, btn_w, btn_h);
                let col = if btn_hover { Color::new(0.25, 0.75, 0.4, 1.0) } else { Color::new(0.18, 0.58, 0.3, 1.0) };
                draw_rectangle(btn_x, btn_y, btn_w, btn_h, col);
                draw_rectangle_lines(btn_x, btn_y, btn_w, btn_h, 1.0, WHITE);
                draw_text(&item.price_display, btn_x + 28.0, btn_y + 27.0, 18.0, WHITE);

                if btn_hover && mouse_clicked {
                    action = ShopAction::BuyItem(item.id.clone());
                }
            }

            cur_y += item_h + 12.0;
        }

        // SECTION 2: COSMETIC THEMES
        cur_y += 20.0;
        draw_text("COSMETIC CELESTIAL THEMES", card_x + 30.0, cur_y, 18.0, Color::new(0.4, 0.9, 1.0, 1.0));
        cur_y += 18.0;

        for item in &catalog.skin_items {
            let is_unlocked = unlocked_skins.contains(&item.id);
            let is_equipped = equipped_skin == item.id;
            let item_h = 65.0;
            let item_w = card_w - 40.0;
            let item_x = card_x + 20.0;

            draw_rectangle(item_x, cur_y, item_w, item_h, Color::new(0.11, 0.09, 0.20, 0.8));
            draw_rectangle_lines(item_x, cur_y, item_w, item_h, 1.0, Color::new(0.3, 0.4, 0.6, 0.4));

            draw_text(&item.title, item_x + 15.0, cur_y + 26.0, 19.0, WHITE);
            draw_text(&item.description, item_x + 15.0, cur_y + 48.0, 13.0, Color::new(0.7, 0.75, 0.85, 0.7));

            let btn_w = 110.0;
            let btn_h = 38.0;
            let btn_x = item_x + item_w - btn_w - 15.0;
            let btn_y = cur_y + 13.0;

            if is_equipped {
                draw_rectangle(btn_x, btn_y, btn_w, btn_h, Color::new(0.2, 0.5, 0.7, 0.7));
                draw_text("EQUIPPED", btn_x + 12.0, btn_y + 25.0, 16.0, WHITE);
            } else if is_unlocked {
                let btn_hover = is_inside(mouse_pos, btn_x, btn_y, btn_w, btn_h);
                draw_rectangle(btn_x, btn_y, btn_w, btn_h, if btn_hover { Color::new(0.35, 0.55, 0.85, 1.0) } else { Color::new(0.25, 0.45, 0.7, 1.0) });
                draw_rectangle_lines(btn_x, btn_y, btn_w, btn_h, 1.0, WHITE);
                draw_text("EQUIP", btn_x + 28.0, btn_y + 25.0, 16.0, WHITE);
                if btn_hover && mouse_clicked {
                    action = ShopAction::EquipSkin(item.id.clone());
                }
            } else {
                let can_afford = stardust >= item.stardust_price;
                let btn_hover = is_inside(mouse_pos, btn_x, btn_y, btn_w, btn_h);
                let col = if can_afford {
                    if btn_hover { Color::new(0.85, 0.6, 0.2, 1.0) } else { Color::new(0.75, 0.5, 0.15, 1.0) }
                } else {
                    Color::new(0.3, 0.3, 0.35, 0.6)
                };
                draw_rectangle(btn_x, btn_y, btn_w, btn_h, col);
                draw_rectangle_lines(btn_x, btn_y, btn_w, btn_h, 1.0, WHITE);
                draw_text(&item.price_display, btn_x + 12.0, btn_y + 25.0, 14.0, WHITE);

                if can_afford && btn_hover && mouse_clicked {
                    action = ShopAction::BuyItem(item.id.clone());
                }
            }

            cur_y += item_h + 10.0;
        }

        action
    }
}

fn is_inside(pos: Vec2, x: f32, y: f32, w: f32, h: f32) -> bool {
    pos.x >= x && pos.x <= x + w && pos.y >= y && pos.y <= y + h
}
