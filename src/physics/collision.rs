use macroquad::prelude::Vec2;
use crate::core::config::{RESTITUTION, STARDUST_PER_FUSION_BASE};
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
    /// Resolves collisions and returns a list of fusions that occurred
    pub fn resolve_collisions(bodies: &mut Vec<CelestialBody>) -> Vec<FusionEvent> {
        let mut fusions = Vec::new();
        let mut to_remove_ids = Vec::new();
        let mut new_bodies = Vec::new();

        let n = bodies.len();
        for i in 0..n {
            if to_remove_ids.contains(&bodies[i].id) {
                continue;
            }

            for j in (i + 1)..n {
                if to_remove_ids.contains(&bodies[j].id) {
                    continue;
                }

                let diff = bodies[j].pos - bodies[i].pos;
                let dist = diff.length();
                let min_dist = bodies[i].radius + bodies[j].radius;

                if dist < min_dist && dist > 0.0001 {
                    // Check if identical tier -> Nuclear Fusion!
                    if bodies[i].tier == bodies[j].tier {
                        if let Some(next_tier) = bodies[i].tier.next_tier() {
                            let midpoint = (bodies[i].pos + bodies[j].pos) * 0.5;
                            // Conserve momentum
                            let total_mass = bodies[i].mass + bodies[j].mass;
                            let combined_vel = (bodies[i].vel * bodies[i].mass + bodies[j].vel * bodies[j].mass) / total_mass;

                            let new_id = (bodies[i].id.wrapping_mul(31) ^ bodies[j].id).wrapping_add(100);
                            let new_body = CelestialBody::new(new_id, next_tier, midpoint, combined_vel * 0.85);

                            let score = next_tier.score_value();
                            let stardust = STARDUST_PER_FUSION_BASE * (next_tier as u32 + 1);

                            fusions.push(FusionEvent {
                                pos: midpoint,
                                new_tier: next_tier,
                                score_awarded: score,
                                stardust_awarded: stardust,
                            });

                            new_bodies.push(new_body);
                            to_remove_ids.push(bodies[i].id);
                            to_remove_ids.push(bodies[j].id);
                            break;
                        }
                    }

                    // Elastic collision resolution
                    let normal = diff / dist;
                    let overlap = min_dist - dist;

                    // Positional separation based on inverse mass
                    let total_inv_mass = 1.0 / bodies[i].mass + 1.0 / bodies[j].mass;
                    let ratio_i = (1.0 / bodies[i].mass) / total_inv_mass;
                    let ratio_j = (1.0 / bodies[j].mass) / total_inv_mass;

                    bodies[i].pos -= normal * (overlap * ratio_i);
                    bodies[j].pos += normal * (overlap * ratio_j);

                    // Impulse velocity resolution
                    let relative_vel = bodies[j].vel - bodies[i].vel;
                    let vel_along_normal = relative_vel.dot(normal);

                    if vel_along_normal < 0.0 {
                        let impulse_mag = -(1.0 + RESTITUTION) * vel_along_normal / total_inv_mass;
                        let impulse = normal * impulse_mag;
                        let mass_i = bodies[i].mass;
                        let mass_j = bodies[j].mass;

                        bodies[i].vel -= impulse / mass_i;
                        bodies[j].vel += impulse / mass_j;
                    }
                }
            }
        }

        // Apply removals and additions
        if !to_remove_ids.is_empty() {
            bodies.retain(|b| !to_remove_ids.contains(&b.id));
            bodies.extend(new_bodies);
        }

        fusions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identical_tier_fusion() {
        let mut bodies = vec![
            CelestialBody::new(1, CelestialTier::Asteroid, Vec2::new(100.0, 100.0), Vec2::new(5.0, 0.0)),
            CelestialBody::new(2, CelestialTier::Asteroid, Vec2::new(105.0, 100.0), Vec2::new(-5.0, 0.0)),
        ];

        let fusions = CollisionEngine::resolve_collisions(&mut bodies);
        assert_eq!(fusions.len(), 1);
        assert_eq!(fusions[0].new_tier, CelestialTier::Moon);
        assert_eq!(bodies.len(), 1);
        assert_eq!(bodies[0].tier, CelestialTier::Moon);
    }
}
