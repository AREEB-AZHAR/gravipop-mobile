use macroquad::prelude::Vec2;
use crate::core::config::{BODY_RESTITUTION, COLLISION_PASSES, STARDUST_PER_FUSION_BASE};
use super::body::CelestialBody;
use super::celestial_tier::CelestialTier;

#[derive(Debug, Clone)]
pub struct FusionEvent {
    pub pos: Vec2,
    pub new_tier: CelestialTier,
    pub score_awarded: u64,
    pub stardust_awarded: u32,
}

pub struct CollisionEngine;

impl CollisionEngine {
    /// Resolve all collisions and return any fusion (merge) events.
    ///
    /// Algorithm:
    ///   1. Merge detection: Any same-tier bodies touching (dist <= contact * 1.02)
    ///      with age >= MIN_FUSION_AGE fuse into the next tier.
    ///   2. Apply merges (retire fused bodies, add next-tier body).
    ///   3. COLLISION_PASSES iterations of elastic separation with lateral rolling
    ///      perturbation for unstable vertical stacks and tangential friction.
    ///   4. Strict container boundary clamping (left, right, bottom) so bodies
    ///      never penetrate the jar floor or walls.
    pub fn resolve_collisions(bodies: &mut Vec<CelestialBody>) -> Vec<FusionEvent> {
        let mut fusions = Vec::new();
        let mut to_remove: Vec<u64> = Vec::new();
        let mut to_add: Vec<CelestialBody> = Vec::new();
        let mut next_id_offset = 0u64;

        // Minimum age (seconds) before a newly fused body can merge again (prevents same-frame multi-merges)
        const MIN_FUSION_AGE: f32 = 0.05;

        // ── Pass 0: Merge detection ───────────────────────────────────────────
        'merge_scan: for i in 0..bodies.len() {
            if to_remove.contains(&bodies[i].id) { continue; }
            for j in (i + 1)..bodies.len() {
                if to_remove.contains(&bodies[j].id) { continue; }

                let diff = bodies[j].pos - bodies[i].pos;
                let dist = diff.length();
                let contact = bodies[i].radius + bodies[j].radius;

                // Touching / colliding + same tier + neither is a brand-new merge → FUSE!
                if dist <= contact * 1.02
                    && bodies[i].tier == bodies[j].tier
                    && bodies[i].age >= MIN_FUSION_AGE
                    && bodies[j].age >= MIN_FUSION_AGE
                {
                    if let Some(next_tier) = bodies[i].tier.next_tier() {
                        let raw_pos = (bodies[i].pos + bodies[j].pos) * 0.5;
                        let r_next = next_tier.radius();

                        // Clamp spawn position inside container bounds
                        let merge_pos = Vec2::new(
                            raw_pos.x.clamp(crate::core::config::JAR_LEFT + r_next, crate::core::config::JAR_RIGHT - r_next),
                            raw_pos.y.min(crate::core::config::JAR_BOTTOM - r_next),
                        );

                        let total_mass = bodies[i].mass + bodies[j].mass;
                        // Conserve momentum, damp velocity so merged body settles gracefully
                        let merged_vel = (bodies[i].vel * bodies[i].mass
                            + bodies[j].vel * bodies[j].mass)
                            / total_mass
                            * 0.40;

                        let new_id = bodies[i]
                            .id
                            .wrapping_mul(2654435761)
                            ^ bodies[j].id
                            ^ next_id_offset;
                        next_id_offset += 1;

                        let mut new_body = CelestialBody::new(new_id, next_tier, merge_pos, merged_vel);
                        // Start at age 0.0 so it cannot fuse again in the exact same instant
                        new_body.age = 0.0;

                        fusions.push(FusionEvent {
                            pos: merge_pos,
                            new_tier: next_tier,
                            score_awarded: next_tier.score_value(),
                            stardust_awarded: STARDUST_PER_FUSION_BASE * (next_tier as u32 + 1),
                        });

                        to_remove.push(bodies[i].id);
                        to_remove.push(bodies[j].id);
                        to_add.push(new_body);
                        continue 'merge_scan;
                    }
                    // Singularity + Singularity: max tier reached, collide normally
                }
            }
        }

        // Apply merges
        if !to_remove.is_empty() {
            bodies.retain(|b| !to_remove.contains(&b.id));
            bodies.extend(to_add);
        }

        // ── Passes 1‥N: Physical separation & Rolling dynamics ───────────────
        for _ in 0..COLLISION_PASSES {
            let n = bodies.len();
            for i in 0..n {
                for j in (i + 1)..n {
                    let (left, right) = bodies.split_at_mut(j);
                    let bi = &mut left[i];
                    let bj = &mut right[0];

                    let diff = bj.pos - bi.pos;
                    let dist = diff.length();
                    let min_dist = bi.radius + bj.radius;

                    if dist < 0.001 {
                        // Coincident bodies — break symmetry with determinism
                        let nudge = if (bi.id ^ bj.id) % 2 == 0 { 1.0 } else { -1.0 };
                        bj.pos.x += nudge * 1.5;
                        bi.pos.x -= nudge * 1.5;
                        continue;
                    }

                    if dist >= min_dist { continue; }

                    let mut normal = diff / dist;
                    let overlap = min_dist - dist;

                    // Unstable vertical equilibrium break (Totem Pole fix):
                    // When spheres are stacked directly above each other, gravity rolls
                    // the upper sphere off the curved shoulder of the lower sphere.
                    let dx_abs = diff.x.abs();
                    if dx_abs < (min_dist * 0.28).max(4.0) && diff.y.abs() > 1.0 {
                        let roll_sign = if diff.x > 0.05 {
                            1.0
                        } else if diff.x < -0.05 {
                            -1.0
                        } else {
                            if (bi.id.wrapping_add(bj.id)) % 2 == 0 { 1.0 } else { -1.0 }
                        };

                        // Deflect collision normal so separation pushes laterally
                        normal.x += 0.24 * roll_sign;
                        normal = normal.normalize();

                        // Transfer lateral roll velocity to upper body
                        if bi.pos.y < bj.pos.y {
                            // bi is on top of bj -> bi rolls away
                            bi.vel.x -= roll_sign * 16.0;
                            bj.vel.x += roll_sign * 8.0;
                        } else {
                            // bj is on top of bi -> bj rolls away
                            bj.vel.x += roll_sign * 16.0;
                            bi.vel.x -= roll_sign * 8.0;
                        }
                    }

                    // Position correction (mass-weighted)
                    let total_mass = bi.mass + bj.mass;
                    let push_i = overlap * (bj.mass / total_mass);
                    let push_j = overlap * (bi.mass / total_mass);
                    bi.pos -= normal * push_i;
                    bj.pos += normal * push_j;

                    // Normal velocity impulse
                    let rel_vel = bj.vel - bi.vel;
                    let v_normal = rel_vel.dot(normal);
                    if v_normal < 0.0 {
                        let e = BODY_RESTITUTION;
                        let inv_sum = 1.0 / bi.mass + 1.0 / bj.mass;
                        let impulse = -(1.0 + e) * v_normal / inv_sum;
                        bi.vel -= normal * (impulse / bi.mass);
                        bj.vel += normal * (impulse / bj.mass);
                    }

                    // Tangential friction (damps excessive sliding, enables realistic rolling settling)
                    let tangent = Vec2::new(-normal.y, normal.x);
                    let v_tangent = rel_vel.dot(tangent);
                    let friction_damping = 0.30;
                    bi.vel += tangent * (v_tangent * friction_damping * (bj.mass / total_mass));
                    bj.vel -= tangent * (v_tangent * friction_damping * (bi.mass / total_mass));
                }
            }
        }

        // ── Container Boundary Enforcement ────────────────────────────────────
        // Clamping here guarantees bodies NEVER protrude through floor or walls
        use crate::core::config::{JAR_LEFT, JAR_RIGHT, JAR_BOTTOM, FLOOR_RESTITUTION, FLOOR_FRICTION, WALL_RESTITUTION};
        for b in bodies.iter_mut() {
            let r = b.radius;

            // Floor
            if b.pos.y + r > JAR_BOTTOM {
                b.pos.y = JAR_BOTTOM - r;
                if b.vel.y > 0.0 {
                    b.vel.y = -b.vel.y * FLOOR_RESTITUTION;
                    b.vel.x *= FLOOR_FRICTION;
                }
            }
            // Left Wall
            if b.pos.x - r < JAR_LEFT {
                b.pos.x = JAR_LEFT + r;
                if b.vel.x < 0.0 {
                    b.vel.x = -b.vel.x * WALL_RESTITUTION;
                }
            }
            // Right Wall
            if b.pos.x + r > JAR_RIGHT {
                b.pos.x = JAR_RIGHT - r;
                if b.vel.x > 0.0 {
                    b.vel.x = -b.vel.x * WALL_RESTITUTION;
                }
            }

            // Micro-velocity sleep when resting on floor
            if b.pos.y + r >= JAR_BOTTOM - 3.0 && b.vel.length_squared() < 36.0 {
                b.vel.x *= 0.85;
                if b.vel.length_squared() < 4.0 {
                    b.vel = Vec2::ZERO;
                }
            }
        }

        fusions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use macroquad::prelude::Vec2;

    #[test]
    fn test_same_tier_merges() {
        let mut bodies = vec![
            {
                let mut b = CelestialBody::new(1, CelestialTier::Asteroid,
                    Vec2::new(100.0, 800.0), Vec2::ZERO);
                b.age = 1.0; // old enough to merge
                b
            },
            {
                let mut b = CelestialBody::new(2, CelestialTier::Asteroid,
                    Vec2::new(118.0, 800.0), Vec2::ZERO); // r1+r2 = 44, dist=18 < 44
                b.age = 1.0;
                b
            },
        ];

        let events = CollisionEngine::resolve_collisions(&mut bodies);
        assert_eq!(events.len(), 1, "should produce one fusion event");
        assert_eq!(events[0].new_tier, CelestialTier::Moon);
        assert_eq!(bodies.len(), 1);
        assert_eq!(bodies[0].tier, CelestialTier::Moon);
    }

    #[test]
    fn test_different_tier_no_merge() {
        let mut bodies = vec![
            {
                let mut b = CelestialBody::new(1, CelestialTier::Asteroid,
                    Vec2::new(100.0, 800.0), Vec2::ZERO);
                b.age = 1.0;
                b
            },
            {
                let mut b = CelestialBody::new(2, CelestialTier::Moon,
                    Vec2::new(118.0, 800.0), Vec2::ZERO);
                b.age = 1.0;
                b
            },
        ];

        let events = CollisionEngine::resolve_collisions(&mut bodies);
        assert_eq!(events.len(), 0, "different tiers should NOT merge");
        assert_eq!(bodies.len(), 2);
    }

    #[test]
    fn test_too_young_no_merge() {
        let mut bodies = vec![
            CelestialBody::new(1, CelestialTier::Asteroid, Vec2::new(100.0, 800.0), Vec2::ZERO),
            CelestialBody::new(2, CelestialTier::Asteroid, Vec2::new(118.0, 800.0), Vec2::ZERO),
        ];
        // age = 0.0, too young

        let events = CollisionEngine::resolve_collisions(&mut bodies);
        assert_eq!(events.len(), 0, "brand-new bodies should not merge immediately");
    }

    #[test]
    fn test_touching_surface_merges() {
        // Asteroid radius is 22.0, so contact is 44.0.
        // At dist = 44.0 (touching at boundary, 0% penetration), it MUST merge!
        let mut bodies = vec![
            {
                let mut b = CelestialBody::new(1, CelestialTier::Asteroid, Vec2::new(200.0, 800.0), Vec2::ZERO);
                b.age = 0.5;
                b
            },
            {
                let mut b = CelestialBody::new(2, CelestialTier::Asteroid, Vec2::new(244.0, 800.0), Vec2::ZERO);
                b.age = 0.5;
                b
            },
        ];

        let events = CollisionEngine::resolve_collisions(&mut bodies);
        assert_eq!(events.len(), 1, "surface touching bodies must merge");
        assert_eq!(bodies.len(), 1);
        assert_eq!(bodies[0].tier, CelestialTier::Moon);
    }

    #[test]
    fn test_boundary_clamping_prevents_penetration() {
        use crate::core::config::JAR_BOTTOM;
        // Body placed below floor
        let mut bodies = vec![
            CelestialBody::new(1, CelestialTier::Moon, Vec2::new(200.0, JAR_BOTTOM + 50.0), Vec2::new(0.0, 100.0)),
        ];
        CollisionEngine::resolve_collisions(&mut bodies);
        assert!(bodies[0].pos.y + bodies[0].radius <= JAR_BOTTOM + 0.01, "body must not penetrate floor");
    }
}

