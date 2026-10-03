use macroquad::prelude::*;
use crate::physics::{CelestialBody, CelestialTier};

pub struct BodyRenderer;

impl BodyRenderer {
    /// Renders a full physical celestial body with dynamic animations, glow, and tier-specific features.
    pub fn draw_body(body: &CelestialBody) {
        Self::render_tier(
            body.pos,
            body.radius,
            body.tier,
            body.rotation,
            body.pulse_phase,
            body.id,
            false,
        );
    }

    /// Renders a compact, scaled preview miniature suitable for HUD cards and reserve slots.
    /// Clamps glow and features within `radius * 1.15` so it never spills over card boundaries or text.
    pub fn draw_preview(tier: CelestialTier, center: Vec2, radius: f32) {
        Self::render_tier(
            center,
            radius,
            tier,
            0.4,
            0.0,
            1,
            true,
        );
    }

    /// Internal unified renderer supporting both full simulation bodies and miniature previews.
    fn render_tier(
        pos: Vec2,
        r: f32,
        tier: CelestialTier,
        rotation: f32,
        pulse_phase: f32,
        _id: u64,
        is_preview: bool,
    ) {
        let primary = tier.primary_color();
        let glow = tier.glow_color();
        let pulse = if is_preview { 1.0 } else { pulse_phase.sin() * 0.08 + 0.96 };

        // ── 0. Rings Behind (Saturn back-half of ring system) ────────────────
        if tier == CelestialTier::RingedGiant {
            let ring_a = r * 1.80;
            let ring_b = r * 0.54;
            let steps = if is_preview { 24 } else { 48 };
            // Draw back arc: angle from PI to 2.0 * PI
            for i in 0..=steps {
                let t = std::f32::consts::PI + (i as f32 / steps as f32) * std::f32::consts::PI;
                let angle = t + rotation * 0.3;
                let rx = angle.cos() * ring_a;
                let ry = angle.sin() * ring_b;
                let dot_r = if is_preview { 2.0 } else { 3.2 };
                // Outer ring A (Golden)
                draw_circle(pos.x + rx, pos.y + ry, dot_r, Color::new(0.96, 0.82, 0.38, 0.65));
                // Inner ring B (Amber)
                draw_circle(pos.x + rx * 0.82, pos.y + ry * 0.82, dot_r * 0.85, Color::new(0.92, 0.68, 0.25, 0.70));
            }
        }

        // ── 1. Atmospheric Glow / Halo ───────────────────────────────────────
        let glow_scale_outer = if is_preview { 1.14 } else { 1.35 * pulse };
        let glow_scale_inner = if is_preview { 1.06 } else { 1.16 * pulse };
        let glow_alpha_outer = if is_preview { 0.18 } else { 0.28 };
        let glow_alpha_inner = if is_preview { 0.35 } else { 0.55 };

        draw_circle(pos.x, pos.y, r * glow_scale_outer, Color::new(glow.r, glow.g, glow.b, glow.a * glow_alpha_outer));
        draw_circle(pos.x, pos.y, r * glow_scale_inner, Color::new(glow.r, glow.g, glow.b, glow.a * glow_alpha_inner));

        // ── 2. Base Sphere & 3D Shading ──────────────────────────────────────
        draw_circle(pos.x, pos.y, r, primary);

        // 3D Spherical Ambient Shadow (bottom-right crescent)
        draw_circle(pos.x + r * 0.22, pos.y + r * 0.22, r * 0.82, Color::new(0.0, 0.0, 0.04, 0.32));
        // 3D Spherical Specular Highlight (top-left dome)
        draw_circle(pos.x - r * 0.26, pos.y - r * 0.26, r * 0.48, Color::new(1.0, 1.0, 1.0, 0.22));
        // Specular glint
        draw_circle(pos.x - r * 0.34, pos.y - r * 0.34, r * 0.14, Color::new(1.0, 1.0, 1.0, 0.42));

        // ── 3. Bespoke Tier-Specific Features ─────────────────────────────────
        match tier {
            CelestialTier::Asteroid => {
                // Rugged Craggy Basalt Surface with Craters & Amber Ore Veins
                // Crater 1 (North-West)
                draw_circle(pos.x - r * 0.32, pos.y - r * 0.20, r * 0.28, Color::new(0.32, 0.30, 0.28, 0.90));
                draw_circle(pos.x - r * 0.34, pos.y - r * 0.22, r * 0.22, Color::new(0.20, 0.18, 0.18, 0.95));
                draw_circle_lines(pos.x - r * 0.32, pos.y - r * 0.20, r * 0.28, 1.2, Color::new(0.72, 0.70, 0.66, 0.65));

                // Crater 2 (South-East)
                draw_circle(pos.x + r * 0.26, pos.y + r * 0.22, r * 0.22, Color::new(0.32, 0.30, 0.28, 0.90));
                draw_circle(pos.x + r * 0.28, pos.y + r * 0.24, r * 0.17, Color::new(0.18, 0.17, 0.17, 0.95));
                draw_circle_lines(pos.x + r * 0.26, pos.y + r * 0.22, r * 0.22, 1.0, Color::new(0.72, 0.70, 0.66, 0.60));

                // Crater 3 (Small micro-crater)
                draw_circle(pos.x - r * 0.12, pos.y + r * 0.38, r * 0.14, Color::new(0.25, 0.24, 0.23, 0.95));

                // Glowing Molten Amber Ore Seam
                let rot_cos = (rotation * 0.4).cos();
                let rot_sin = (rotation * 0.4).sin();
                let v1 = Vec2::new(pos.x + rot_cos * r * 0.1, pos.y + rot_sin * r * 0.1);
                let v2 = Vec2::new(pos.x + rot_cos * r * 0.55, pos.y - rot_sin * r * 0.5);
                draw_line(v1.x, v1.y, v2.x, v2.y, if is_preview { 1.5 } else { 2.2 }, Color::new(1.0, 0.72, 0.22, 0.85));
            }

            CelestialTier::Moon => {
                // Silvery Lunar Maria & Crater Basins
                // Lunar Maria (Dark Basalt Seas)
                draw_circle(pos.x - r * 0.28, pos.y - r * 0.15, r * 0.34, Color::new(0.50, 0.60, 0.70, 0.38));
                draw_circle(pos.x + r * 0.18, pos.y + r * 0.20, r * 0.30, Color::new(0.50, 0.60, 0.70, 0.32));
                draw_circle(pos.x - r * 0.10, pos.y + r * 0.32, r * 0.22, Color::new(0.48, 0.58, 0.68, 0.35));

                // Impact Crater with Ejecta Rays (Tycho analog)
                let c_x = pos.x + r * 0.22;
                let c_y = pos.y - r * 0.24;
                draw_circle(c_x, c_y, r * 0.16, Color::new(0.92, 0.96, 1.0, 0.85));
                draw_circle(c_x + 1.0, c_y + 1.0, r * 0.11, Color::new(0.42, 0.50, 0.60, 0.90));
                // Ray lines
                for k in 0..4 {
                    let ray_ang = k as f32 * std::f32::consts::FRAC_PI_2 + 0.3;
                    let rx = ray_ang.cos() * r * 0.32;
                    let ry = ray_ang.sin() * r * 0.32;
                    draw_line(c_x, c_y, c_x + rx, c_y + ry, 1.0, Color::new(1.0, 1.0, 1.0, 0.45));
                }

                // Crisp White Terminator Limb
                draw_circle_lines(pos.x - r * 0.05, pos.y - r * 0.05, r * 0.96, 1.5, Color::new(1.0, 1.0, 1.0, 0.40));
            }

            CelestialTier::Terrestrial => {
                // Earth: Vibrant Azure Oceans, Emerald Continents, Ice Caps & Clouds
                let spin = rotation * 0.3;

                // Continents (Emerald green landmasses)
                let c1_x = pos.x + (spin).cos() * r * 0.35;
                let c1_y = pos.y + (spin).sin() * r * 0.20;
                draw_circle(c1_x, c1_y, r * 0.32, Color::new(0.18, 0.78, 0.38, 0.88));
                draw_circle(c1_x + r * 0.12, c1_y - r * 0.08, r * 0.22, Color::new(0.24, 0.85, 0.42, 0.85));

                let c2_x = pos.x - (spin).cos() * r * 0.30;
                let c2_y = pos.y - (spin).sin() * r * 0.25;
                draw_circle(c2_x, c2_y, r * 0.26, Color::new(0.16, 0.72, 0.34, 0.85));

                // Polar Ice Caps
                draw_circle(pos.x, pos.y - r * 0.82, r * 0.26, Color::new(0.94, 0.98, 1.0, 0.90));
                draw_circle(pos.x, pos.y + r * 0.84, r * 0.22, Color::new(0.92, 0.97, 1.0, 0.85));

                // Swirling Atmospheric Cloud Bands (White Whorls)
                let cloud_spin = spin * 1.4;
                let cl_x1 = pos.x + (cloud_spin).cos() * r * 0.42;
                let cl_y1 = pos.y - r * 0.15;
                draw_circle(cl_x1, cl_y1, r * 0.20, Color::new(1.0, 1.0, 1.0, 0.65));
                draw_circle(cl_x1 + r * 0.15, cl_y1 + r * 0.08, r * 0.14, Color::new(1.0, 1.0, 1.0, 0.55));

                let cl_x2 = pos.x - (cloud_spin).cos() * r * 0.38;
                let cl_y2 = pos.y + r * 0.22;
                draw_circle(cl_x2, cl_y2, r * 0.18, Color::new(1.0, 1.0, 1.0, 0.60));

                // Atmospheric Rayleigh scattering halo on edge
                draw_circle_lines(pos.x, pos.y, r * 0.98, 1.8, Color::new(0.40, 0.85, 1.0, 0.55));
            }

            CelestialTier::GasGiant => {
                // Jupiter: Vibrant Horizontal Storm Bands & The Great Red Spot
                let bw = r * 0.88;
                let thick = if is_preview { 2.2 } else { 3.8 };

                // Band 1: North Polar Belt (Deep Rust)
                draw_line(pos.x - bw * 0.70, pos.y - r * 0.52, pos.x + bw * 0.70, pos.y - r * 0.52, thick * 0.85, Color::new(0.75, 0.32, 0.12, 0.80));
                // Band 2: North Temperate Belt (Warm Cream / Gold)
                draw_line(pos.x - bw * 0.88, pos.y - r * 0.26, pos.x + bw * 0.88, pos.y - r * 0.26, thick, Color::new(0.98, 0.82, 0.45, 0.75));
                // Band 3: Equatorial Zone (Bright Amber Sand)
                draw_line(pos.x - bw * 0.96, pos.y - r * 0.02, pos.x + bw * 0.96, pos.y - r * 0.02, thick * 1.2, Color::new(1.0, 0.68, 0.25, 0.70));
                // Band 4: South Temperate Belt (Deep Crimson)
                draw_line(pos.x - bw * 0.90, pos.y + r * 0.26, pos.x + bw * 0.90, pos.y + r * 0.26, thick, Color::new(0.82, 0.22, 0.12, 0.85));
                // Band 5: South Polar Belt
                draw_line(pos.x - bw * 0.72, pos.y + r * 0.54, pos.x + bw * 0.72, pos.y + r * 0.54, thick * 0.85, Color::new(0.70, 0.28, 0.10, 0.80));

                // The Great Red Spot Storm Vortex
                let spot_x = pos.x + (rotation * 0.25).sin() * r * 0.40;
                let spot_y = pos.y + r * 0.28;
                let spot_rx = r * 0.26;
                let spot_ry = r * 0.17;
                draw_circle(spot_x, spot_y, spot_rx, Color::new(0.85, 0.15, 0.12, 0.95));
                draw_circle(spot_x, spot_y, spot_ry, Color::new(1.0, 0.45, 0.30, 0.95));
                draw_circle(spot_x - 1.0, spot_y - 1.0, spot_ry * 0.5, Color::new(1.0, 0.85, 0.75, 0.90));
            }

            CelestialTier::RingedGiant => {
                // Saturn: Golden Body with Equatorial Ring Division
                // Subtle planetary cloud belts
                let thick = if is_preview { 1.8 } else { 2.8 };
                draw_line(pos.x - r * 0.82, pos.y - r * 0.22, pos.x + r * 0.82, pos.y - r * 0.22, thick, Color::new(0.92, 0.70, 0.20, 0.50));
                draw_line(pos.x - r * 0.92, pos.y + r * 0.05, pos.x + r * 0.92, pos.y + r * 0.05, thick * 1.1, Color::new(1.0, 0.90, 0.50, 0.55));
                draw_line(pos.x - r * 0.80, pos.y + r * 0.28, pos.x + r * 0.80, pos.y + r * 0.28, thick, Color::new(0.90, 0.65, 0.18, 0.50));

                // Front Ring Arc (drawn over bottom half of the planet)
                let ring_a = r * 1.80;
                let ring_b = r * 0.54;
                let steps = if is_preview { 24 } else { 48 };
                for i in 0..=steps {
                    let t = (i as f32 / steps as f32) * std::f32::consts::PI;
                    let angle = t + rotation * 0.3;
                    let rx = angle.cos() * ring_a;
                    let ry = angle.sin() * ring_b;
                    let dot_r = if is_preview { 2.0 } else { 3.2 };
                    // Outer Ring A
                    draw_circle(pos.x + rx, pos.y + ry, dot_r, Color::new(1.0, 0.88, 0.45, 0.85));
                    // Inner Ring B
                    draw_circle(pos.x + rx * 0.82, pos.y + ry * 0.82, dot_r * 0.85, Color::new(0.95, 0.75, 0.30, 0.90));
                }

                // Planet Shadow onto front ring
                draw_circle(pos.x + r * 0.15, pos.y + r * 0.32, r * 0.28, Color::new(0.0, 0.0, 0.05, 0.35));
            }

            CelestialTier::IceGiant => {
                // Crystalline Turquoise Glacial World with Faceted Sheets & Auroral Poles
                // Crystalline Surface Facets (Geometric ice fractures)
                let thick = if is_preview { 1.5 } else { 2.2 };
                let col_facet = Color::new(0.75, 0.95, 1.0, 0.65);
                draw_line(pos.x - r * 0.55, pos.y - r * 0.35, pos.x + r * 0.20, pos.y - r * 0.50, thick, col_facet);
                draw_line(pos.x + r * 0.20, pos.y - r * 0.50, pos.x + r * 0.55, pos.y - r * 0.15, thick, col_facet);
                draw_line(pos.x + r * 0.55, pos.y - r * 0.15, pos.x + r * 0.30, pos.y + r * 0.40, thick, col_facet);
                draw_line(pos.x + r * 0.30, pos.y + r * 0.40, pos.x - r * 0.40, pos.y + r * 0.35, thick, col_facet);
                draw_line(pos.x - r * 0.40, pos.y + r * 0.35, pos.x - r * 0.55, pos.y - r * 0.35, thick, col_facet);

                // Central crystalline vertex star
                draw_line(pos.x, pos.y - r * 0.20, pos.x + r * 0.25, pos.y + r * 0.10, thick * 0.9, col_facet);
                draw_line(pos.x, pos.y - r * 0.20, pos.x - r * 0.25, pos.y + r * 0.10, thick * 0.9, col_facet);

                // Glowing Neon-Cyan Auroral Crowns at magnetic poles
                let aur_pulse = (pulse_phase * 2.0).sin().abs() * 0.25 + 0.75;
                draw_circle(pos.x, pos.y - r * 0.78, r * 0.24, Color::new(0.35, 1.0, 0.95, 0.60 * aur_pulse));
                draw_circle(pos.x, pos.y + r * 0.78, r * 0.24, Color::new(0.35, 1.0, 0.95, 0.55 * aur_pulse));
            }

            CelestialTier::RedDwarf => {
                // Convective Molten Star with Granule Texture & Fiery Prominences
                // Convective solar granules
                for k in 0..5 {
                    let a = k as f32 * 1.25 + rotation * 0.5;
                    let gx = pos.x + a.cos() * r * 0.42;
                    let gy = pos.y + a.sin() * r * 0.42;
                    draw_circle(gx, gy, r * 0.22, Color::new(1.0, 0.65, 0.15, 0.65));
                    draw_circle(gx, gy, r * 0.12, Color::new(1.0, 0.90, 0.35, 0.75));
                }

                // Coronal Flares and Looping Prominences
                let flare_count = if is_preview { 5 } else { 8 };
                for i in 0..flare_count {
                    let a = (i as f32 * std::f32::consts::TAU / flare_count as f32) + rotation * 1.8;
                    let flare_len = r + (if is_preview { 4.0 } else { 8.0 + (a * 2.5).sin().abs() * 6.0 });
                    let fx = pos.x + a.cos() * flare_len;
                    let fy = pos.y + a.sin() * flare_len;
                    let f_size = if is_preview { 2.5 } else { 4.5 };
                    draw_circle(fx, fy, f_size, Color::new(1.0, 0.45, 0.10, 0.85));
                    draw_circle(fx, fy, f_size * 0.6, Color::new(1.0, 0.85, 0.25, 0.95));
                }
            }

            CelestialTier::BlueSupergiant => {
                // Thermonuclear Supergiant with Radiant Coronal Light Rays & Blinding Core
                // Blinding white thermonuclear core
                draw_circle(pos.x, pos.y, r * 0.45, Color::new(0.92, 0.98, 1.0, 0.95));
                draw_circle(pos.x, pos.y, r * 0.22, WHITE);

                // Radiant Starburst Coronal Rays (8 Cardinal Beams)
                let ray_count = 8;
                for i in 0..ray_count {
                    let a = (i as f32 * std::f32::consts::TAU / ray_count as f32) + rotation * 0.6;
                    let ray_len = r + (if is_preview { 5.0 } else { 12.0 });
                    let rx = pos.x + a.cos() * ray_len;
                    let ry = pos.y + a.sin() * ray_len;
                    draw_line(pos.x, pos.y, rx, ry, if is_preview { 1.5 } else { 2.6 }, Color::new(0.40, 0.92, 1.0, 0.75));
                }

                // Plasma discharge filaments
                for k in 0..4 {
                    let a = k as f32 * 1.57 + rotation * 1.2;
                    let px = pos.x + a.cos() * r * 0.75;
                    let py = pos.y + a.sin() * r * 0.75;
                    draw_circle(px, py, if is_preview { 2.0 } else { 3.5 }, Color::new(1.0, 1.0, 1.0, 0.85));
                }
            }

            CelestialTier::Pulsar => {
                // Relativistic Magnetar with Dipole Field Loops & Sweeping Dual Radiation Jets
                // Dense violet core
                draw_circle(pos.x, pos.y, r * 0.55, Color::new(0.95, 0.45, 1.0, 0.95));
                draw_circle(pos.x, pos.y, r * 0.28, Color::new(1.0, 0.90, 1.0, 1.0));

                // Equatorial Magnetic Field Loops
                let mag_r = r * 0.85;
                draw_circle_lines(pos.x, pos.y, mag_r, 1.4, Color::new(0.70, 0.35, 1.0, 0.55));

                // Sweeping Collimated Relativistic Particle Jets (Opposite poles)
                let jet_ang = rotation * 3.2;
                let jet_len = r * (if is_preview { 1.4 } else { 2.1 });
                let jx = jet_ang.cos() * jet_len;
                let jy = jet_ang.sin() * jet_len;

                // Jet 1 (North Pole)
                draw_line(pos.x, pos.y, pos.x + jx, pos.y + jy, if is_preview { 2.5 } else { 4.5 }, Color::new(0.92, 0.40, 1.0, 0.85));
                draw_line(pos.x, pos.y, pos.x + jx, pos.y + jy, if is_preview { 1.2 } else { 2.0 }, WHITE);
                draw_circle(pos.x + jx, pos.y + jy, if is_preview { 2.5 } else { 4.0 }, Color::new(1.0, 0.80, 1.0, 0.95));

                // Jet 2 (South Pole)
                draw_line(pos.x, pos.y, pos.x - jx, pos.y - jy, if is_preview { 2.5 } else { 4.5 }, Color::new(0.92, 0.40, 1.0, 0.85));
                draw_line(pos.x, pos.y, pos.x - jx, pos.y - jy, if is_preview { 1.2 } else { 2.0 }, WHITE);
                draw_circle(pos.x - jx, pos.y - jy, if is_preview { 2.5 } else { 4.0 }, Color::new(1.0, 0.80, 1.0, 0.95));
            }

            CelestialTier::Singularity => {
                // Pitch-Black Event Horizon + Relativistic Accretion Disk + Gravitational Lensing
                // Swirling Accretion Disk (Iridescent Purple & Golden matter)
                let disk_a = r * 1.40;
                let disk_b = r * 0.50;
                let steps = if is_preview { 24 } else { 36 };
                for i in 0..steps {
                    let a = (i as f32 / steps as f32) * std::f32::consts::TAU + rotation * 2.5;
                    let dx = a.cos() * disk_a;
                    let dy = a.sin() * disk_b;
                    // Doppler boosting: matter rotating towards viewer is brighter
                    let doppler = (a.sin() * 0.35 + 0.65).clamp(0.2, 1.0);
                    let col = if i % 2 == 0 {
                        Color::new(0.95 * doppler, 0.35 * doppler, 0.90 * doppler, 0.85)
                    } else {
                        Color::new(1.0 * doppler, 0.80 * doppler, 0.25 * doppler, 0.80)
                    };
                    draw_circle(pos.x + dx, pos.y + dy, if is_preview { 2.2 } else { 3.8 }, col);
                }

                // Einstein Gravitational Lensing Photon Ring
                draw_circle_lines(pos.x, pos.y, r * 0.88, if is_preview { 1.5 } else { 2.4 }, Color::new(1.0, 0.88, 1.0, 0.90));

                // Absolute Pitch-Black Event Horizon (Inside Schwarzschild radius)
                draw_circle(pos.x, pos.y, r * 0.80, Color::new(0.01, 0.01, 0.03, 1.0));
            }
        }

        // ── 4. Crisp Celestial Rim Outline ───────────────────────────────────
        let rim_alpha = if is_preview { 0.50 } else { 0.40 };
        draw_circle_lines(pos.x, pos.y, r, 1.6, Color::new(1.0, 1.0, 1.0, rim_alpha));
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
