use crate::core::config::{VIRTUAL_HEIGHT, VIRTUAL_WIDTH};
use macroquad::prelude::*;

pub struct AdOverlay;

impl AdOverlay {
    /// The provider owns its creative and close controls. This only covers the
    /// paused canvas behind it; there is no timer or local reward shortcut.
    pub fn draw(font: Option<&Font>) {
        draw_rectangle(
            0.0,
            0.0,
            VIRTUAL_WIDTH,
            VIRTUAL_HEIGHT,
            Color::new(0.02, 0.02, 0.06, 0.98),
        );
        for (text, y, size) in [
            ("ADVERTISEMENT", VIRTUAL_HEIGHT * 0.44, 30u16),
            ("Your game is paused.", VIRTUAL_HEIGHT * 0.50, 22),
            (
                "Close the ad to return to your game.",
                VIRTUAL_HEIGHT * 0.55,
                20,
            ),
        ] {
            let dim = measure_text(text, font, size, 1.0);
            draw_text_ex(
                text,
                (VIRTUAL_WIDTH - dim.width) * 0.5,
                y,
                TextParams {
                    font,
                    font_size: size,
                    color: WHITE,
                    ..Default::default()
                },
            );
        }
    }
}
