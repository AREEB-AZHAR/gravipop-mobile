use super::body::CelestialBody;
use crate::core::config::{
    BODY_RESTITUTION, COLLISION_PASSES, FLOOR_FRICTION, FLOOR_RESTITUTION, JAR_BOTTOM, JAR_LEFT,
    JAR_RIGHT, JAR_TOP_LINE, MERGE_COOLDOWN, STARDUST_PER_FUSION_BASE, WALL_RESTITUTION,
};
use macroquad::prelude::Vec2;

#[derive(Debug, Clone)]
pub struct FusionEvent {
    pub pos: Vec2,
    pub new_tier: super::celestial_tier::CelestialTier,
    pub score_awarded: u64,
    pub stardust_awarded: u32,
}

pub struct CollisionEngine;

impl CollisionEngine {
    /// Solve solid contacts, contain bodies, then fuse eligible touching pairs.
    /// Merges use the shared cooldown, so a new body cannot chain instantly.
    pub fn resolve_collisions(bodies: &mut Vec<CelestialBody>) -> Vec<FusionEvent> {
        let mut fusions = Vec::new();
        for _ in 0..COLLISION_PASSES {
            for i in 0..bodies.len() {
                for j in (i + 1)..bodies.len() {
                    let (left, right) = bodies.split_at_mut(j);
                    let (a, b) = (&mut left[i], &mut right[0]);
                    let delta = b.pos - a.pos;
                    let distance = delta.length();
                    let minimum = a.radius + b.radius;
                    if distance >= minimum {
                        continue;
                    }

                    let normal = if distance > 0.001 {
                        delta / distance
                    } else {
                        let angle = ((a.id ^ b.id) % 16) as f32 * std::f32::consts::TAU / 16.0;
                        Vec2::new(angle.cos(), angle.sin())
                    };
                    let overlap = minimum - distance;
                    let inverse_a = 1.0 / a.mass.max(0.001);
                    let inverse_b = 1.0 / b.mass.max(0.001);
                    let inverse_sum = inverse_a + inverse_b;

                    // Small slop and partial correction avoid visible jitter in stacks.
                    let correction = (overlap - 0.15).max(0.0) * 0.82 / inverse_sum;
                    a.pos -= normal * (correction * inverse_a);
                    b.pos += normal * (correction * inverse_b);

                    let relative = b.vel - a.vel;
                    let normal_speed = relative.dot(normal);
                    let mut normal_impulse = 0.0;
                    if normal_speed < 0.0 {
                        normal_impulse = -(1.0 + BODY_RESTITUTION) * normal_speed / inverse_sum;
                        a.vel -= normal * (normal_impulse * inverse_a);
                        b.vel += normal * (normal_impulse * inverse_b);
                    }

                    // Friction only removes relative tangent velocity; it never adds
                    // the arbitrary sideways kicks that made vertical stacks unstable.
                    let tangent = relative - normal * normal_speed;
                    let tangent_speed = tangent.length();
                    if tangent_speed > 0.001 {
                        let friction_impulse = (tangent_speed / inverse_sum)
                            .min(normal_impulse * 0.32 + overlap * 0.08);
                        let impulse = tangent / tangent_speed * friction_impulse;
                        a.vel += impulse * inverse_a;
                        b.vel -= impulse * inverse_b;
                    }
                }
            }

            for body in bodies.iter_mut() {
                let r = body.radius;
                if body.pos.x - r < JAR_LEFT {
                    body.pos.x = JAR_LEFT + r;
                    if body.vel.x < 0.0 {
                        body.vel.x = -body.vel.x * WALL_RESTITUTION;
                    }
                }
                if body.pos.x + r > JAR_RIGHT {
                    body.pos.x = JAR_RIGHT - r;
                    if body.vel.x > 0.0 {
                        body.vel.x = -body.vel.x * WALL_RESTITUTION;
                    }
                }
                if body.pos.y + r > JAR_BOTTOM {
                    body.pos.y = JAR_BOTTOM - r;
                    if body.vel.y > 0.0 {
                        body.vel.y = -body.vel.y * FLOOR_RESTITUTION;
                        body.vel.x *= FLOOR_FRICTION;
                    }
                }
            }
        }

        let mut removed = vec![false; bodies.len()];
        let mut added = Vec::new();
        'pairs: for i in 0..bodies.len() {
            if removed[i] {
                continue;
            }
            for j in (i + 1)..bodies.len() {
                if removed[j] {
                    continue;
                }
                let a = &bodies[i];
                let b = &bodies[j];
                if a.tier != b.tier || a.age < MERGE_COOLDOWN || b.age < MERGE_COOLDOWN {
                    continue;
                }
                if (b.pos - a.pos).length_squared() > (a.radius + b.radius + 0.25).powi(2) {
                    continue;
                }
                let Some(tier) = a.tier.next_tier() else {
                    continue;
                };

                let radius = tier.radius();
                let pos = ((a.pos + b.pos) * 0.5).clamp(
                    Vec2::new(JAR_LEFT + radius, JAR_TOP_LINE + radius),
                    Vec2::new(JAR_RIGHT - radius, JAR_BOTTOM - radius),
                );
                let mass = a.mass + b.mass;
                let velocity = (a.vel * a.mass + b.vel * b.mass) / mass.max(0.001) * 0.40;
                let id = a.id.wrapping_mul(2654435761) ^ b.id ^ (fusions.len() as u64);
                let mut merged = CelestialBody::new(id, tier, pos, velocity);
                merged.age = 0.0;
                added.push(merged);
                removed[i] = true;
                removed[j] = true;
                fusions.push(FusionEvent {
                    pos,
                    new_tier: tier,
                    score_awarded: tier.score_value(),
                    stardust_awarded: STARDUST_PER_FUSION_BASE * (tier as u32 + 1),
                });
                continue 'pairs;
            }
        }
        if removed.iter().any(|v| *v) {
            let mut idx = 0;
            bodies.retain(|_| {
                let keep = !removed[idx];
                idx += 1;
                keep
            });
            bodies.extend(added);
        }

        // Quiet bodies touching the floor settle without accumulating tiny jitter.
        for body in bodies.iter_mut() {
            if body.pos.y + body.radius >= JAR_BOTTOM - 0.5 && body.vel.length_squared() < 9.0 {
                body.vel = Vec2::ZERO;
            }
        }
        fusions
    }
}

/// Fixed-step wrapper makes contacts consistent across variable display rates.
pub struct PhysicsWorld {
    accumulator: f32,
}

impl Default for PhysicsWorld {
    fn default() -> Self {
        Self { accumulator: 0.0 }
    }
}

impl PhysicsWorld {
    pub fn step(&mut self, bodies: &mut Vec<CelestialBody>, frame_dt: f32) -> Vec<FusionEvent> {
        const STEP: f32 = 1.0 / 120.0;
        const MAX_STEPS: usize = 8;
        self.accumulator = (self.accumulator + frame_dt.clamp(0.0, STEP * MAX_STEPS as f32))
            .min(STEP * MAX_STEPS as f32);
        let mut events = Vec::new();
        let mut steps = 0;
        while self.accumulator >= STEP && steps < MAX_STEPS {
            for body in bodies.iter_mut() {
                body.update(STEP);
            }
            events.extend(CollisionEngine::resolve_collisions(bodies));
            self.accumulator -= STEP;
            steps += 1;
        }
        events
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physics::CelestialTier;

    fn body(id: u64, tier: CelestialTier, x: f32, y: f32, age: f32) -> CelestialBody {
        let mut b = CelestialBody::new(id, tier, Vec2::new(x, y), Vec2::ZERO);
        b.age = age;
        b
    }

    #[test]
    fn equal_tier_surface_contact_merges_after_cooldown() {
        let mut bodies = vec![
            body(1, CelestialTier::Asteroid, 200.0, 800.0, MERGE_COOLDOWN),
            body(
                2,
                CelestialTier::Asteroid,
                200.0 + CelestialTier::Asteroid.radius() * 2.0,
                800.0,
                MERGE_COOLDOWN,
            ),
        ];
        let events = CollisionEngine::resolve_collisions(&mut bodies);
        assert_eq!(events.len(), 1);
        assert_eq!(bodies.len(), 1);
        assert_eq!(bodies[0].tier, CelestialTier::Moon);
    }

    #[test]
    fn young_bodies_do_not_merge_early() {
        let mut bodies = vec![
            body(
                1,
                CelestialTier::Asteroid,
                200.0,
                800.0,
                MERGE_COOLDOWN - 0.01,
            ),
            body(2, CelestialTier::Asteroid, 220.0, 800.0, 1.0),
        ];
        assert!(CollisionEngine::resolve_collisions(&mut bodies).is_empty());
        assert_eq!(bodies.len(), 2);
    }

    #[test]
    fn different_tiers_separate_without_fusing() {
        let mut bodies = vec![
            body(1, CelestialTier::Asteroid, 200.0, 800.0, 1.0),
            body(2, CelestialTier::Moon, 220.0, 800.0, 1.0),
        ];
        assert!(CollisionEngine::resolve_collisions(&mut bodies).is_empty());
        assert_eq!(bodies.len(), 2);
        let distance = (bodies[0].pos - bodies[1].pos).length();
        assert!(distance >= bodies[0].radius + bodies[1].radius - 0.5);
    }

    #[test]
    fn boundaries_contain_bodies() {
        let mut bodies = vec![body(
            1,
            CelestialTier::Moon,
            JAR_LEFT - 80.0,
            JAR_BOTTOM + 60.0,
            1.0,
        )];
        CollisionEngine::resolve_collisions(&mut bodies);
        assert!(bodies[0].pos.x - bodies[0].radius >= JAR_LEFT - 0.01);
        assert!(bodies[0].pos.y + bodies[0].radius <= JAR_BOTTOM + 0.01);
    }

    #[test]
    fn physics_world_accumulates_partial_frames() {
        let mut world = PhysicsWorld::default();
        let mut bodies = vec![CelestialBody::new(
            1,
            CelestialTier::Asteroid,
            Vec2::new(300.0, 200.0),
            Vec2::ZERO,
        )];
        world.step(&mut bodies, 1.0 / 240.0);
        assert_eq!(bodies[0].pos.y, 200.0);
        world.step(&mut bodies, 1.0 / 240.0);
        assert!(bodies[0].pos.y > 200.0);
    }
}
