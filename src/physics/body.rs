use macroquad::prelude::Vec2;
use super::celestial_tier::CelestialTier;
use crate::core::config::*;

#[derive(Debug, Clone)]
pub struct CelestialBody {
    pub id: u64,
    pub tier: CelestialTier,
    pub pos: Vec2,
    pub vel: Vec2,
    pub radius: f32,
    pub mass: f32,
    /// How long this body has existed. New bodies cannot merge for MERGE_COOLDOWN seconds.
    pub age: f32,
    /// Visual pulse phase for glow animation.
    pub pulse_phase: f32,
    pub rotation: f32,
    pub rotation_speed: f32,
}

impl CelestialBody {
    pub fn new(id: u64, tier: CelestialTier, pos: Vec2, vel: Vec2) -> Self {
        Self {
            id,
            tier,
            pos,
            vel,
            radius: tier.radius(),
            mass: tier.mass(),
            age: 0.0,
            pulse_phase: (id as f32 * 1.7) % std::f32::consts::TAU,
            rotation: 0.0,
            rotation_speed: ((id as f32 * 0.37) % 2.0 - 1.0) * 0.6,
        }
    }

    /// Advance physics: gravity, integrate, wall & floor collision.
    pub fn update(&mut self, dt: f32) {
        // Downward gravity
        self.vel.y += GRAVITY_ACCEL * dt;

        // Integrate
        self.pos += self.vel * dt;

        // Visual
        self.age += dt;
        self.pulse_phase += dt * 2.5;
        self.rotation += self.rotation_speed * dt;

        let r = self.radius;

        // Left wall
        if self.pos.x - r < JAR_LEFT {
            self.pos.x = JAR_LEFT + r;
            self.vel.x = self.vel.x.abs() * WALL_RESTITUTION;
            self.vel.y *= 0.99;
        }
        // Right wall
        if self.pos.x + r > JAR_RIGHT {
            self.pos.x = JAR_RIGHT - r;
            self.vel.x = -self.vel.x.abs() * WALL_RESTITUTION;
            self.vel.y *= 0.99;
        }
        // Floor
        if self.pos.y + r > JAR_BOTTOM {
            self.pos.y = JAR_BOTTOM - r;
            self.vel.y = -self.vel.y.abs() * FLOOR_RESTITUTION;
            self.vel.x *= FLOOR_FRICTION;
        }
    }

    /// True when this body has essentially come to rest.
    pub fn is_settled(&self) -> bool {
        self.age > 0.4 && self.vel.length_squared() < 400.0 // ~20 px/s
    }

    /// True when the body (settled) is above the game-over danger line.
    pub fn above_danger_line(&self) -> bool {
        self.is_settled() && self.pos.y - self.radius < JAR_TOP_LINE
    }
}
