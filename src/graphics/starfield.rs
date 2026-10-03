use macroquad::prelude::*;
use crate::core::config::{CORE_RADIUS, EVENT_HORIZON_RADIUS, VIRTUAL_HEIGHT, VIRTUAL_WIDTH};
use quad_rand::gen_range;

struct Star {
    pos: Vec2,
    speed: f32,
    size: f32,
    brightness: f32,
    twinkle_phase: f32,
}

pub struct Starfield {
    stars: Vec<Star>,
    accretion_rotation: f32,
}

impl Starfield {
    pub fn new() -> Self {
        let mut stars = Vec::with_capacity(120);
        for _ in 0..120 {
            stars.push(Star {
                pos: Vec2::new(gen_range(0.0, VIRTUAL_WIDTH), gen_range(0.0, VIRTUAL_HEIGHT)),
                speed: gen_range(3.0, 18.0),
                size: gen_range(1.0, 2.5),
                brightness: gen_range(0.3, 0.95),
                twinkle_phase: gen_range(0.0, std::f32::consts::PI * 2.0),
            });
        }

        Self {
            stars,
            accretion_rotation: 0.0,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.accretion_rotation += dt * 0.8;
        for star in self.stars.iter_mut() {
            star.pos.y += star.speed * dt;
            if star.pos.y > VIRTUAL_HEIGHT {
                star.pos.y = 0.0;
                star.pos.x = gen_range(0.0, VIRTUAL_WIDTH);
            }
            star.twinkle_phase += dt * 3.5;
        }
    }

    pub fn draw(&self, center: Vec2, danger_ratio: f32) {
        // Deep space cosmic background
        clear_background(Color::new(0.04, 0.03, 0.09, 1.0));

        // Draw ambient space nebulae
        draw_circle(center.x - 140.0, center.y - 180.0, 260.0, Color::new(0.18, 0.08, 0.32, 0.12));
        draw_circle(center.x + 160.0, center.y + 140.0, 240.0, Color::new(0.05, 0.20, 0.35, 0.14));

        // Draw stars
        for star in &self.stars {
            let twinkle = (star.twinkle_phase.sin() * 0.35 + 0.65) * star.brightness;
            let col = Color::new(0.9, 0.95, 1.0, twinkle);
            draw_circle(star.pos.x, star.pos.y, star.size, col);
        }

        // Event Horizon Perimeter (Safe orbit limit)
        let mut perimeter_color = Color::new(0.35, 0.50, 0.85, 0.25);
        if danger_ratio > 0.01 {
            // Pulse red when danger is active!
            let pulse = (get_time() as f32 * 12.0).sin().abs();
            perimeter_color = Color::new(1.0, 0.2, 0.2, 0.3 + danger_ratio * 0.45 * pulse);
            draw_circle_lines(center.x, center.y, EVENT_HORIZON_RADIUS + 2.0, 3.5, perimeter_color);
        } else {
            draw_circle_lines(center.x, center.y, EVENT_HORIZON_RADIUS, 1.8, perimeter_color);
        }

        // Cosmic Singularity (Central Core)
        // Outer accretion disk glow
        for i in 0..16 {
            let angle = self.accretion_rotation + (i as f32 * std::f32::consts::PI / 8.0);
            let radius_offset = (angle * 3.0).sin() * 6.0;
            let p_x = center.x + angle.cos() * (CORE_RADIUS + 18.0 + radius_offset);
            let p_y = center.y + angle.sin() * (CORE_RADIUS + 18.0 + radius_offset);
            draw_circle(p_x, p_y, 4.0, Color::new(0.85, 0.35, 1.0, 0.45));
        }

        // Inner glowing corona
        draw_circle(center.x, center.y, CORE_RADIUS + 8.0, Color::new(0.55, 0.15, 0.95, 0.35));
        draw_circle(center.x, center.y, CORE_RADIUS + 2.0, Color::new(0.85, 0.45, 1.0, 0.7));

        // Event horizon black hole center void
        draw_circle(center.x, center.y, CORE_RADIUS, Color::new(0.01, 0.01, 0.03, 1.0));
        draw_circle_lines(center.x, center.y, CORE_RADIUS, 2.0, Color::new(1.0, 0.8, 1.0, 0.6));
    }
}
