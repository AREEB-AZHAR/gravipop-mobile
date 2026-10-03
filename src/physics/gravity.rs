use macroquad::prelude::Vec2;
use crate::core::config::{CORE_RADIUS, DAMPING, EVENT_HORIZON_RADIUS, GRAVITATIONAL_CONSTANT};
use super::body::CelestialBody;

pub struct GravityEngine {
    pub center: Vec2,
}

impl GravityEngine {
    pub fn new(center: Vec2) -> Self {
        Self { center }
    }

    pub fn apply_forces(&self, bodies: &mut [CelestialBody], dt: f32) {
        let n = bodies.len();

        // 1. Central Singularity Gravity
        for body in bodies.iter_mut() {
            let to_center = self.center - body.pos;
            let dist = to_center.length();

            if dist > CORE_RADIUS + body.radius * 0.5 {
                // Modified power law for smooth, stable Keplerian orbital play
                let force_mag = GRAVITATIONAL_CONSTANT / (dist.powf(1.15) + 60.0);
                let force_dir = to_center / dist;
                body.vel += force_dir * force_mag * dt;
            } else {
                // Gentle repulsion from central singularity core
                let normal = (body.pos - self.center).normalize();
                body.vel = normal * 120.0;
                body.pos = self.center + normal * (CORE_RADIUS + body.radius * 0.6);
            }

            // Orbital damping
            body.vel *= DAMPING;

            // Check if outside Event Horizon
            body.is_outside = dist > EVENT_HORIZON_RADIUS;
            if body.is_outside {
                body.outside_timer += dt;
            } else {
                body.outside_timer = (body.outside_timer - dt * 2.0).max(0.0);
            }
        }

        // 2. Inter-Body Gravity (Gentle mutual pull encourages natural coalescence)
        for i in 0..n {
            for j in (i + 1)..n {
                let diff = bodies[j].pos - bodies[i].pos;
                let dist = diff.length();
                if dist > (bodies[i].radius + bodies[j].radius) && dist < 180.0 {
                    let force = (diff / dist) * (350.0 / (dist + 30.0)) * dt;
                    bodies[i].vel += force;
                    bodies[j].vel -= force;
                }
            }
        }
    }

    /// Predicts trajectory points for the slingshot reticle
    pub fn predict_trajectory(&self, start_pos: Vec2, initial_vel: Vec2, steps: usize, dt: f32) -> Vec<Vec2> {
        let mut pos = start_pos;
        let mut vel = initial_vel;
        let mut points = Vec::with_capacity(steps);

        for _ in 0..steps {
            let to_center = self.center - pos;
            let dist = to_center.length();

            if dist > CORE_RADIUS + 15.0 {
                let force_mag = GRAVITATIONAL_CONSTANT / (dist.powf(1.15) + 60.0);
                let force_dir = to_center / dist;
                vel += force_dir * force_mag * dt;
            }
            vel *= DAMPING;
            pos += vel * dt;
            points.push(pos);

            if dist < CORE_RADIUS {
                break;
            }
        }
        points
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physics::CelestialTier;

    #[test]
    fn test_gravity_pulls_towards_center() {
        let center = Vec2::new(360.0, 640.0);
        let engine = GravityEngine::new(center);

        let mut bodies = vec![CelestialBody::new(
            1,
            CelestialTier::Asteroid,
            Vec2::new(360.0, 400.0), // Above center
            Vec2::ZERO,
        )];

        engine.apply_forces(&mut bodies, 0.1);
        // Acceleration should pull downwards towards center (positive Y velocity)
        assert!(bodies[0].vel.y > 0.0);
    }

    #[test]
    fn test_trajectory_prediction_length() {
        let center = Vec2::new(360.0, 640.0);
        let engine = GravityEngine::new(center);
        let trajectory = engine.predict_trajectory(
            Vec2::new(360.0, 1100.0),
            Vec2::new(50.0, -150.0),
            20,
            0.03,
        );
        assert!(!trajectory.is_empty());
        assert!(trajectory.len() <= 20);
    }
}
