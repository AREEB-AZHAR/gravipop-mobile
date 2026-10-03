use macroquad::prelude::*;
use crate::core::config::{VIRTUAL_HEIGHT, VIRTUAL_WIDTH};
use quad_rand::gen_range;

struct Star {
    pos: Vec2,
    speed: f32,
    size: f32,
    twinkle_phase: f32,
    brightness: f32,
}

pub struct Starfield {
    stars: Vec<Star>,
}

impl Default for Starfield {
    fn default() -> Self {
        Self::new()
    }
}

impl Starfield {
    pub fn new() -> Self {
        let stars = (0..140)
            .map(|_| Star {
                pos: Vec2::new(
                    gen_range(0.0, VIRTUAL_WIDTH),
                    gen_range(0.0, VIRTUAL_HEIGHT),
                ),
                speed: gen_range(4.0, 22.0),
                size: gen_range(0.8, 2.4),
                brightness: gen_range(0.30, 1.0),
                twinkle_phase: gen_range(0.0, std::f32::consts::TAU),
            })
            .collect();

        Self { stars }
    }

    pub fn update(&mut self, dt: f32) {
        for s in self.stars.iter_mut() {
            s.pos.y += s.speed * dt;
            if s.pos.y > VIRTUAL_HEIGHT {
                s.pos.y = -2.0;
                s.pos.x = gen_range(0.0, VIRTUAL_WIDTH);
            }
            s.twinkle_phase += dt * 3.2;
        }
    }

    /// `danger_ratio` 0‥1: how close to game-over (tints background red).
    pub fn draw(&self, danger_ratio: f32) {
        // Background gradient: deep space
        let r = 0.04 + danger_ratio * 0.08;
        let g = 0.03;
        let b = 0.09 - danger_ratio * 0.04;
        clear_background(Color::new(r, g, b, 1.0));

        // Soft nebula blobs for atmosphere
        draw_circle(160.0, 340.0, 280.0, Color::new(0.14, 0.06, 0.28, 0.10));
        draw_circle(560.0, 900.0, 260.0, Color::new(0.05, 0.16, 0.30, 0.12));
        draw_circle(360.0, 640.0, 320.0, Color::new(0.08, 0.04, 0.18, 0.08));

        // Stars
        for s in &self.stars {
            let twinkle = (s.twinkle_phase.sin() * 0.30 + 0.70) * s.brightness;
            draw_circle(s.pos.x, s.pos.y, s.size, Color::new(0.90, 0.95, 1.0, twinkle));
        }
    }
}
