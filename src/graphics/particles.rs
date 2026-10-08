use macroquad::prelude::*;
use quad_rand::gen_range;

#[derive(Clone, Copy)]
pub struct Particle {
    pub pos: Vec2,
    pub vel: Vec2,
    pub color: Color,
    pub size: f32,
    pub lifetime: f32,
    pub max_lifetime: f32,
}

#[derive(Clone)]
pub struct FloatingText {
    pub text: String,
    pub pos: Vec2,
    pub vel: Vec2,
    pub color: Color,
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub font_size: f32,
    pub width: f32,
}

pub struct ParticleEngine {
    pub particles: Vec<Particle>,
    pub floating_texts: Vec<FloatingText>,
}

impl Default for ParticleEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ParticleEngine {
    pub fn new() -> Self {
        Self {
            particles: Vec::with_capacity(512),
            floating_texts: Vec::with_capacity(32),
        }
    }

    pub fn spawn_fusion_burst(&mut self, pos: Vec2, base_color: Color, tier_index: usize) {
        let count = 25 + tier_index * 8;
        for _ in 0..count {
            let angle = gen_range(0.0, std::f32::consts::PI * 2.0);
            let speed = gen_range(40.0, 220.0 + tier_index as f32 * 30.0);
            let vel = Vec2::new(angle.cos(), angle.sin()) * speed;
            let size = gen_range(2.0, 5.5 + tier_index as f32 * 0.8);
            let max_lifetime = gen_range(0.4, 0.9);

            let mut color = base_color;
            color.a = gen_range(0.7, 1.0);

            self.particles.push(Particle {
                pos,
                vel,
                color,
                size,
                lifetime: max_lifetime,
                max_lifetime,
            });
        }
    }

    pub fn spawn_trail(&mut self, pos: Vec2, color: Color) {
        let angle = gen_range(0.0, std::f32::consts::PI * 2.0);
        let speed = gen_range(5.0, 25.0);
        let vel = Vec2::new(angle.cos(), angle.sin()) * speed;
        self.particles.push(Particle {
            pos,
            vel,
            color: Color::new(color.r, color.g, color.b, 0.4),
            size: gen_range(2.0, 3.8),
            lifetime: 0.35,
            max_lifetime: 0.35,
        });
    }

    pub fn add_floating_text_with_width(&mut self, text: String, pos: Vec2, color: Color, font_size: f32, width: f32) {
        self.floating_texts.push(FloatingText {
            text,
            pos,
            vel: Vec2::new(gen_range(-15.0, 15.0), -65.0),
            color,
            lifetime: 1.1,
            max_lifetime: 1.1,
            font_size,
            width,
        });
    }

    pub fn add_floating_text(&mut self, text: String, pos: Vec2, color: Color, font_size: f32) {
        let approx_width = text.len() as f32 * font_size * 0.52;
        self.add_floating_text_with_width(text, pos, color, font_size, approx_width);
    }

    pub fn update(&mut self, dt: f32) {
        // Update particles
        for p in self.particles.iter_mut() {
            p.pos += p.vel * dt;
            p.vel *= 0.96; // drag
            p.lifetime -= dt;
        }
        self.particles.retain(|p| p.lifetime > 0.0);

        // Update floating text
        for t in self.floating_texts.iter_mut() {
            t.pos += t.vel * dt;
            t.vel.y *= 0.95;
            t.lifetime -= dt;
        }
        self.floating_texts.retain(|t| t.lifetime > 0.0);
    }

    pub fn draw(&self, font: Option<&Font>) {
        for p in &self.particles {
            let alpha = (p.lifetime / p.max_lifetime).clamp(0.0, 1.0);
            let mut draw_color = p.color;
            draw_color.a *= alpha;
            let current_size = p.size * alpha;
            draw_circle(p.pos.x, p.pos.y, current_size, draw_color);
        }

        for t in &self.floating_texts {
            let alpha = (t.lifetime / t.max_lifetime).clamp(0.0, 1.0);
            let mut col = t.color;
            col.a *= alpha;
            let sz = t.font_size.round() as u16;
            let px = (t.pos.x - t.width * 0.5).round();
            let py = t.pos.y.round();
            // Soft drop shadow for razor-sharp readability against glowing celestial bodies
            draw_text_ex(
                &t.text,
                px + 1.5,
                py + 1.5,
                TextParams {
                    font,
                    font_size: sz,
                    color: Color::new(0.0, 0.0, 0.0, col.a * 0.75),
                    ..Default::default()
                },
            );
            draw_text_ex(
                &t.text,
                px,
                py,
                TextParams {
                    font,
                    font_size: sz,
                    color: col,
                    ..Default::default()
                },
            );
        }
    }
}
