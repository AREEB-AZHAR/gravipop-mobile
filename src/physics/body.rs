use macroquad::prelude::Vec2;
use super::celestial_tier::CelestialTier;

#[derive(Debug, Clone)]
pub struct CelestialBody {
    pub id: u64,
    pub tier: CelestialTier,
    pub pos: Vec2,
    pub vel: Vec2,
    pub radius: f32,
    pub mass: f32,
    pub fusing: bool,
    pub fuse_timer: f32,
    pub pulse_phase: f32,
    pub rotation: f32,
    pub rotation_speed: f32,
    pub is_outside: bool,
    pub outside_timer: f32,
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
            fusing: false,
            fuse_timer: 0.0,
            pulse_phase: 0.0,
            rotation: 0.0,
            rotation_speed: (id as f32 % 3.0 - 1.5) * 0.8,
            is_outside: false,
            outside_timer: 0.0,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.pos += self.vel * dt;
        self.pulse_phase += dt * 3.0;
        self.rotation += self.rotation_speed * dt;

        if self.fusing {
            self.fuse_timer -= dt;
        }
    }
}
