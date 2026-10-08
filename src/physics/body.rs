use super::celestial_tier::CelestialTier;
use crate::core::config::*;
use macroquad::prelude::Vec2;

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
    /// True while falling from the top dispenser into the jar. Transit bodies never trigger overflow.
    pub in_drop_transit: bool,
}

impl CelestialBody {
    pub fn new(id: u64, tier: CelestialTier, pos: Vec2, vel: Vec2) -> Self {
        let in_drop_transit = pos.y < JAR_TOP_LINE;
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
            in_drop_transit,
        }
    }

    /// Advance the body's free motion. Contact resolution is handled centrally
    /// by `CollisionEngine` so bodies are not clamped twice per frame.
    pub fn update(&mut self, dt: f32) {
        // Downward gravity
        self.vel.y += GRAVITY_ACCEL * dt;

        // Integrate
        self.pos += self.vel * dt;

        // Visual
        self.age += dt;
        self.pulse_phase += dt * 2.5;
        self.rotation += self.rotation_speed * dt;

        // Once the entire body has entered below the rim line, drop transit is finished
        if self.in_drop_transit && self.pos.y - self.radius >= JAR_TOP_LINE {
            self.in_drop_transit = false;
        }
    }

    /// True when this body has essentially come to rest.
    pub fn is_settled(&self) -> bool {
        self.age > 0.3 && self.vel.length_squared() < 3600.0 // ~60 px/s
    }

    /// True when the body is overflowing above the game-over danger line.
    /// Dropping bodies in transit (including bodies sliding down jar walls) NEVER trigger overflow.
    /// Only bodies that have landed/settled on a stack or are pinned at the rim trigger overflow.
    pub fn is_overflowing(&self) -> bool {
        // 1. A body currently in drop transit falling from the dispenser is NEVER overflowing.
        if self.in_drop_transit {
            return false;
        }

        // 2. Must breach above the danger rim line.
        if self.pos.y - self.radius >= JAR_TOP_LINE {
            return false;
        }

        // 3. Falling downward into the jar at significant speed is transit, not an overflow.
        if self.vel.y > 60.0 {
            return false;
        }

        // 4. Settled or pinned against the rim/wall:
        // - Standard calm check: overall speed < 60 px/s after settling window.
        // - Edge-wall / stack pinned check: even if vibrating horizontally (vel.x ~ 30 px/s)
        //   against the jar walls, its vertical velocity is near zero (|vel.y| < 40 px/s)
        //   while resting above the rim.
        let is_calm = self.vel.length_squared() < 3600.0;
        let is_wall_pinned = self.vel.y.abs() < 40.0;

        self.age > 0.25 && (is_calm || is_wall_pinned)
    }

    /// Backward-compatible alias checking overflow.
    pub fn above_danger_line(&self) -> bool {
        self.is_overflowing()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dropped_planet_sliding_down_wall_never_triggers_overflow() {
        let r = CelestialTier::Moon.radius();
        let mut moon = CelestialBody::new(
            1,
            CelestialTier::Moon,
            Vec2::new(JAR_RIGHT - r, DROP_Y),
            Vec2::new(0.0, 140.0),
        );
        assert!(moon.in_drop_transit);
        assert!(!moon.is_overflowing());

        // Simulate falling along the wall while above the rim
        for _ in 0..15 {
            moon.update(1.0 / 60.0);
            // Even if touching the wall at the rim, it's still in transit
            if moon.pos.y < JAR_TOP_LINE {
                assert!(moon.in_drop_transit);
                assert!(!moon.is_overflowing());
            }
        }
    }

    #[test]
    fn landed_planet_stacked_at_edge_triggers_overflow() {
        let r = CelestialTier::Moon.radius();
        let mut moon = CelestialBody::new(
            2,
            CelestialTier::Moon,
            Vec2::new(JAR_RIGHT - r, JAR_TOP_LINE - 10.0),
            Vec2::new(25.0, 0.0), // Wall vibration with 0 vertical speed
        );
        moon.in_drop_transit = false; // Landed on stack
        moon.age = 0.5;

        assert!(moon.is_overflowing());
    }
}

