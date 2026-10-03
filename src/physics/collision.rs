use macroquad::prelude::Vec2;
use crate::core::config::{BODY_RESTITUTION, COLLISION_PASSES, MERGE_COOLDOWN, STARDUST_PER_FUSION_BASE};
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
    ///   1. One pass to find same-tier pairs and schedule merges.
    ///   2. Apply merges (remove two old bodies, add one new body).
    ///   3. COLLISION_PASSES iterations of elastic separation for remaining bodies.
    pub fn resolve_collisions(bodies: &mut Vec<CelestialBody>) -> Vec<FusionEvent> {
        let mut fusions = Vec::new();
        let mut to_remove: Vec<u64> = Vec::new();
        let mut to_add: Vec<CelestialBody> = Vec::new();
        let mut next_id_offset = 0u64;

        // ── Pass 0: Merge detection ───────────────────────────────────────────
        'merge_scan: for i in 0..bodies.len() {
            if to_remove.contains(&bodies[i].id) { continue; }
            for j in (i + 1)..bodies.len() {
                if to_remove.contains(&bodies[j].id) { continue; }

                let diff = bodies[j].pos - bodies[i].pos;
                let dist = diff.length();
                let contact = bodies[i].radius + bodies[j].radius;

                // Overlapping + same tier + both old enough → merge!
                if dist < contact * 0.92
                    && bodies[i].tier == bodies[j].tier
                    && bodies[i].age > MERGE_COOLDOWN
                    && bodies[j].age > MERGE_COOLDOWN
                {
                    if let Some(next_tier) = bodies[i].tier.next_tier() {
                        let merge_pos = (bodies[i].pos + bodies[j].pos) * 0.5;
                        let total_mass = bodies[i].mass + bodies[j].mass;
                        // Conserve momentum, dampen so merged body settles faster
                        let merged_vel = (bodies[i].vel * bodies[i].mass
                            + bodies[j].vel * bodies[j].mass)
                            / total_mass
                            * 0.35;

                        let new_id = bodies[i]
                            .id
                            .wrapping_mul(2654435761)
                            ^ bodies[j].id
                            ^ next_id_offset;
                        next_id_offset += 1;

                        let mut new_body = CelestialBody::new(new_id, next_tier, merge_pos, merged_vel);
                        // Give it a short "born" state so it can't immediately re-merge
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
                    // Singularity + Singularity: nothing higher, but still collide normally
                }
            }
        }

        // Apply merges
        if !to_remove.is_empty() {
            bodies.retain(|b| !to_remove.contains(&b.id));
            bodies.extend(to_add);
        }

        // ── Passes 1‥N: Physical separation ──────────────────────────────────
        for _ in 0..COLLISION_PASSES {
            let n = bodies.len();
            for i in 0..n {
                for j in (i + 1)..n {
                    // Split borrow trick: safe because i < j
                    let (left, right) = bodies.split_at_mut(j);
                    let bi = &mut left[i];
                    let bj = &mut right[0];

                    let diff = bj.pos - bi.pos;
                    let dist = diff.length();
                    if dist < 0.001 {
                        // Exactly overlapping — push apart along a stable axis
                        bj.pos.x += 0.5;
                        bi.pos.x -= 0.5;
                        continue;
                    }
                    let min_dist = bi.radius + bj.radius;
                    if dist >= min_dist { continue; }

                    let normal = diff / dist;
                    let overlap = min_dist - dist;

                    // Position correction (mass-weighted)
                    let total_mass = bi.mass + bj.mass;
                    let push_i = overlap * (bj.mass / total_mass);
                    let push_j = overlap * (bi.mass / total_mass);
                    bi.pos -= normal * push_i;
                    bj.pos += normal * push_j;

                    // Velocity impulse
                    let rel_vel = bj.vel - bi.vel;
                    let v_normal = rel_vel.dot(normal);
                    if v_normal < 0.0 {
                        let e = BODY_RESTITUTION;
                        let inv_sum = 1.0 / bi.mass + 1.0 / bj.mass;
                        let impulse = -(1.0 + e) * v_normal / inv_sum;
                        bi.vel -= normal * (impulse / bi.mass);
                        bj.vel += normal * (impulse / bj.mass);
                    }
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
}
