use macroquad::prelude::*;
use crate::physics::{CelestialBody, CelestialTier};

pub struct BodyRenderer;

impl BodyRenderer {
    pub fn draw_body(body: &CelestialBody) {
        let pos = body.pos;
        let r = body.radius;
        let tier = body.tier;
        let primary = tier.primary_color();
        let glow = tier.glow_color();

        // 1. Soft atmospheric glow aura
        let pulse = body.pulse_phase.sin() * 0.1 + 0.95;
        draw_circle(pos.x, pos.y, r * 1.45 * pulse, Color::new(glow.r, glow.g, glow.b, glow.a * 0.35));
        draw_circle(pos.x, pos.y, r * 1.2 * pulse, Color::new(glow.r, glow.g, glow.b, glow.a * 0.7));

        // 2. Base planet body sphere
        draw_circle(pos.x, pos.y, r, primary);

        // 3. Pseudo-3D sphere shading (top-left highlight, bottom-right shadow)
        draw_circle(pos.x - r * 0.28, pos.y - r * 0.28, r * 0.55, Color::new(1.0, 1.0, 1.0, 0.22));
        draw_circle(pos.x + r * 0.25, pos.y + r * 0.25, r * 0.65, Color::new(0.0, 0.0, 0.0, 0.25));

        // 4. Tier-Specific Features
        match tier {
            CelestialTier::Asteroid => {
                // Surface craters
                draw_circle(pos.x - r * 0.3, pos.y + r * 0.2, r * 0.22, Color::new(0.4, 0.42, 0.46, 0.8));
                draw_circle(pos.x + r * 0.2, pos.y - r * 0.3, r * 0.18, Color::new(0.4, 0.42, 0.46, 0.8));
            }
            CelestialTier::Moon => {
                draw_circle(pos.x - r * 0.2, pos.y + r * 0.15, r * 0.25, Color::new(0.5, 0.65, 0.75, 0.45));
                draw_circle(pos.x + r * 0.3, pos.y - r * 0.2, r * 0.18, Color::new(0.5, 0.65, 0.75, 0.45));
            }
            CelestialTier::Terrestrial => {
                // Green continents
                draw_circle(pos.x - r * 0.1, pos.y - r * 0.2, r * 0.35, Color::new(0.2, 0.85, 0.35, 0.6));
                draw_circle(pos.x + r * 0.2, pos.y + r * 0.2, r * 0.3, Color::new(0.2, 0.85, 0.35, 0.6));
                // White cloud swirl
                draw_circle(pos.x - r * 0.2, pos.y + r * 0.25, r * 0.18, Color::new(1.0, 1.0, 1.0, 0.4));
            }
            CelestialTier::GasGiant => {
                // Atmospheric storm bands
                draw_line(pos.x - r * 0.85, pos.y - r * 0.3, pos.x + r * 0.85, pos.y - r * 0.3, 3.5, Color::new(0.8, 0.3, 0.1, 0.6));
                draw_line(pos.x - r * 0.95, pos.y, pos.x + r * 0.95, pos.y, 4.0, Color::new(1.0, 0.7, 0.3, 0.5));
                draw_line(pos.x - r * 0.85, pos.y + r * 0.3, pos.x + r * 0.85, pos.y + r * 0.3, 3.5, Color::new(0.8, 0.3, 0.1, 0.6));
            }
            CelestialTier::RingedGiant => {
                // Golden planetary ring ellipse
                for i in 0..24 {
                    let angle = (i as f32 * std::f32::consts::PI / 12.0) + body.rotation;
                    let rx = angle.cos() * r * 1.6;
                    let ry = angle.sin() * r * 0.48;
                    draw_circle(pos.x + rx, pos.y + ry, 2.5, Color::new(1.0, 0.88, 0.45, 0.7));
                }
            }
            CelestialTier::IceGiant => {
                // Crystalline facets
                draw_line(pos.x - r * 0.5, pos.y - r * 0.5, pos.x + r * 0.5, pos.y + r * 0.5, 2.0, Color::new(0.7, 0.9, 1.0, 0.5));
                draw_line(pos.x + r * 0.5, pos.y - r * 0.5, pos.x - r * 0.5, pos.y + r * 0.5, 2.0, Color::new(0.7, 0.9, 1.0, 0.5));
            }
            CelestialTier::RedDwarf => {
                // Solar flares
                for i in 0..6 {
                    let a = (i as f32 * std::f32::consts::PI / 3.0) + body.rotation * 2.0;
                    let fx = pos.x + a.cos() * (r + 7.0);
                    let fy = pos.y + a.sin() * (r + 7.0);
                    draw_circle(fx, fy, 4.5, Color::new(1.0, 0.6, 0.1, 0.7));
                }
            }
            CelestialTier::BlueSupergiant => {
                // Plasma flares
                for i in 0..8 {
                    let a = (i as f32 * std::f32::consts::PI / 4.0) + body.rotation * 2.5;
                    let fx = pos.x + a.cos() * (r + 9.0);
                    let fy = pos.y + a.sin() * (r + 9.0);
                    draw_circle(fx, fy, 5.0, Color::new(0.4, 0.95, 1.0, 0.8));
                }
            }
            CelestialTier::Pulsar => {
                // Spinning radiation beam jets
                let a = body.rotation * 3.5;
                let beam_len = r * 2.2;
                draw_line(pos.x - a.cos() * beam_len, pos.y - a.sin() * beam_len,
                          pos.x + a.cos() * beam_len, pos.y + a.sin() * beam_len, 4.5, Color::new(0.9, 0.4, 1.0, 0.85));
            }
            CelestialTier::Singularity => {
                // Accretion swirl
                for i in 0..12 {
                    let a = (i as f32 * std::f32::consts::PI / 6.0) + body.rotation * 4.0;
                    let sx = pos.x + a.cos() * (r * 1.3);
                    let sy = pos.y + a.sin() * (r * 1.3);
                    draw_circle(sx, sy, 4.0, Color::new(1.0, 0.5, 0.9, 0.75));
                }
            }
        }

        // 5. Crisp rim outline
        draw_circle_lines(pos.x, pos.y, r, 1.6, Color::new(1.0, 1.0, 1.0, 0.4));
    }

    /// Renders the slingshot aiming line and trajectory dots
    pub fn draw_slingshot_trajectory(launch_pos: Vec2, drag_pos: Vec2, trajectory: &[Vec2], tier: CelestialTier) {
        let pull_vec = launch_pos - drag_pos;
        let pull_len = pull_vec.length();

        // 1. Rubber band / sling tether
        let tension_ratio = (pull_len / 150.0).clamp(0.0, 1.0);
        let band_color = Color::new(0.3 + tension_ratio * 0.7, 0.9 - tension_ratio * 0.5, 1.0 - tension_ratio * 0.5, 0.85);
        draw_line(launch_pos.x, launch_pos.y, drag_pos.x, drag_pos.y, 3.5, band_color);

        // 2. Trajectory prediction dots
        for (i, pt) in trajectory.iter().enumerate() {
            let alpha = 1.0 - (i as f32 / trajectory.len() as f32);
            let dot_color = Color::new(1.0, 0.9, 0.4, alpha * 0.75);
            let size = 3.5 * alpha + 1.0;
            draw_circle(pt.x, pt.y, size, dot_color);
        }

        // 3. Body preview at launch position
        let preview_body = CelestialBody::new(0, tier, launch_pos, Vec2::ZERO);
        Self::draw_body(&preview_body);
    }
}
