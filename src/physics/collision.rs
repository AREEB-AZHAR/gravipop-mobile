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
    /// Fuse eligible touching pairs of identical celestial tier.
    fn fuse_eligible_pairs(bodies: &mut Vec<CelestialBody>, fusions: &mut Vec<FusionEvent>) -> bool {
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
                let touch_dist = a.radius + b.radius + 0.35;
                if (b.pos - a.pos).length_squared() > touch_dist * touch_dist {
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
        let any_removed = removed.iter().any(|v| *v);
        if any_removed {
            let mut idx = 0;
            bodies.retain(|_| {
                let keep = !removed[idx];
                idx += 1;
                keep
            });
            bodies.extend(added);
        }
        any_removed
    }

    /// Solve solid contacts, contain bodies, and fuse eligible touching pairs.
    /// Connecting pairs are checked FIRST before applying any collision or nudge forces.
    pub fn resolve_collisions(bodies: &mut Vec<CelestialBody>) -> Vec<FusionEvent> {
        let mut fusions = Vec::new();

        // 1. Connection-First Fusion Check:
        // When a planet drops or contacts a matching partner, immediately fuse them
        // BEFORE applying any physical separation or apex break nudges.
        Self::fuse_eligible_pairs(bodies, &mut fusions);

        for _ in 0..COLLISION_PASSES {
            for i in 0..bodies.len() {
                for j in (i + 1)..bodies.len() {
                    let (left, right) = bodies.split_at_mut(j);
                    let (a, b) = (&mut left[i], &mut right[0]);
                    let mut delta = b.pos - a.pos;
                    let distance = delta.length();
                    let minimum = a.radius + b.radius;
                    if distance >= minimum {
                        continue;
                    }

                    // Identify vertical relationship: which body is top and which is bottom
                    let (top_is_b, top_radius, bottom_radius) = if b.pos.y < a.pos.y {
                        (true, b.radius, a.radius)
                    } else {
                        (false, a.radius, b.radius)
                    };

                    // Unstable Equilibrium Break: A large planet cannot balance on the sharp apex of a smaller body.
                    // Strictly applies ONLY when tiers differ (a.tier != b.tier) and top body is distinctly larger.
                    // Equal-tier planets (which are meant to fuse on contact) must NEVER be nudged!
                    if a.tier != b.tier && delta.x.abs() < 3.5 && (top_radius >= bottom_radius * 1.15) {
                        let sign = if (a.id ^ b.id) % 2 == 0 { 1.0 } else { -1.0 };
                        delta.x = if delta.x.abs() < 0.1 { sign * 3.0 } else { delta.x.signum() * 3.0 };
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

                    // Positional separation with slight slop to prevent jitter
                    let correction = (overlap - 0.12).max(0.0) * 0.82 / inverse_sum;
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

                    // Curvature Roll-Off Dynamics: Downward gravity along the curved contact slope
                    // exerts a lateral rolling torque (F_roll = g * sin(theta) * cos(theta)).
                    // Large bodies roll off small bodies instead of magically perching on top.
                    // Strictly applies ONLY when tiers differ so matching planets do not roll away from each other.
                    let (norm_to_top, top_inv, bot_inv) = if top_is_b {
                        (normal, inverse_b, inverse_a)
                    } else {
                        (-normal, inverse_a, inverse_b)
                    };
                    let vertical_contact = (-norm_to_top.y).max(0.0);
                    if a.tier != b.tier && vertical_contact > 0.10 {
                        let size_multiplier = (top_radius / bottom_radius).max(1.0).min(3.2);
                        let roll_component = norm_to_top.x * vertical_contact;
                        let roll_impulse = 52.0 * roll_component * size_multiplier;
                        if top_is_b {
                            b.vel.x += roll_impulse * (top_inv / inverse_sum);
                            a.vel.x -= roll_impulse * 0.35 * (bot_inv / inverse_sum);
                        } else {
                            a.vel.x += roll_impulse * (top_inv / inverse_sum);
                            b.vel.x -= roll_impulse * 0.35 * (bot_inv / inverse_sum);
                        }
                    }

                    // Rolling friction: Allow spheres to roll smoothly past each other into resting pockets
                    let tangent = relative - normal * normal_speed;
                    let tangent_speed = tangent.length();
                    if tangent_speed > 0.001 {
                        let friction_impulse = (tangent_speed / inverse_sum)
                            .min(normal_impulse * 0.12 + overlap * 0.04);
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

        // 2. Secondary Fusion Pass: Capture any bodies that contacted during collision solver iterations
        Self::fuse_eligible_pairs(bodies, &mut fusions);

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
    use crate::core::config::DROP_Y;
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
        assert_eq!(events[0].score_awarded, CelestialTier::Moon.score_value());
    }

    #[test]
    fn dropping_or_landing_a_planet_never_awards_points_without_a_merge() {
        for tier in CelestialTier::ALL {
            let mut world = PhysicsWorld::default();
            let mut bodies = vec![CelestialBody::new(1, tier, Vec2::new(360.0, DROP_Y), Vec2::new(0.0, 140.0))];
            for _ in 0..300 {
                let events = world.step(&mut bodies, 1.0 / 60.0);
                assert!(events.is_empty(), "{:?} generated points without a pair to merge", tier);
            }
            assert_eq!(bodies.len(), 1);
            assert_eq!(bodies[0].tier, tier);
        }
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

    #[test]
    fn large_planet_rolls_off_small_planet_unstable_apex() {
        let small_radius = CelestialTier::Asteroid.radius();
        let large_radius = CelestialTier::GasGiant.radius();
        // Place small asteroid at bottom, and massive Gas Giant directly vertically atop it
        let mut bodies = vec![
            body(1, CelestialTier::Asteroid, 300.0, 800.0, 1.0),
            body(2, CelestialTier::GasGiant, 300.0, 800.0 - small_radius - large_radius + 1.0, 1.0),
        ];
        let mut world = PhysicsWorld::default();
        // Simulate several physics steps
        for _ in 0..60 {
            world.step(&mut bodies, 1.0 / 60.0);
        }
        // The top gas giant must have rolled off horizontally rather than statically perching atop the asteroid
        let delta_x = (bodies[1].pos.x - bodies[0].pos.x).abs();
        assert!(
            delta_x > 8.0,
            "Large planet should roll off small planet; delta_x was only {}",
            delta_x
        );
    }

    #[test]
    fn equal_tier_drop_connects_and_merges_without_sideways_nudge() {
        let radius = CelestialTier::Moon.radius();
        // Two Moons vertically aligned, one touching the other
        let mut bodies = vec![
            body(10, CelestialTier::Moon, 360.0, 800.0, 1.0),
            body(11, CelestialTier::Moon, 360.0, 800.0 - radius * 2.0 + 1.0, 1.0),
        ];
        let events = CollisionEngine::resolve_collisions(&mut bodies);
        assert_eq!(events.len(), 1, "Touching equal tier bodies must merge immediately");
        assert_eq!(bodies.len(), 1);
        assert_eq!(bodies[0].tier, CelestialTier::Terrestrial);
        // The merged body's X position must remain aligned (never pushed sideways by nudge)
        assert!((bodies[0].pos.x - 360.0).abs() < 0.1, "Merged body must remain centered");
    }
}
