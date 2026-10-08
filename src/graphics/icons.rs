use macroquad::prelude::*;
use std::f32::consts::PI;

/// Procedural vector icon drawing helpers.
/// Completely eliminates missing font glyphs, tofu boxes, or encoding bugs across platforms.
/// Draw a five-pointed star centered at (cx, cy)
pub fn draw_vector_star(cx: f32, cy: f32, outer_radius: f32, color: Color) {
    let inner_radius = outer_radius * 0.42;
    let points = 5;
    let step = PI / points as f32;
    let mut vertices: Vec<Vec2> = Vec::with_capacity(points * 2);

    for i in 0..(points * 2) {
        let r = if i % 2 == 0 { outer_radius } else { inner_radius };
        let angle = i as f32 * step - PI * 0.5;
        vertices.push(Vec2::new(cx + angle.cos() * r, cy + angle.sin() * r));
    }

    // Draw triangles from center to each edge
    let center = Vec2::new(cx, cy);
    for i in 0..vertices.len() {
        let next_i = (i + 1) % vertices.len();
        draw_triangle(center, vertices[i], vertices[next_i], color);
    }
}

/// Draw a sparkling 4-point cosmic diamond / stardust gem
pub fn draw_vector_gem(cx: f32, cy: f32, size: f32, color: Color) {
    let half = size * 0.5;
    let quarter = size * 0.22;

    // Diamond core
    draw_triangle(Vec2::new(cx, cy - half), Vec2::new(cx - quarter, cy), Vec2::new(cx + quarter, cy), color);
    draw_triangle(Vec2::new(cx, cy + half), Vec2::new(cx - quarter, cy), Vec2::new(cx + quarter, cy), color);

    // Cross glint
    let glow = Color::new(color.r, color.g, color.b, color.a * 0.7);
    draw_line(cx - half * 1.3, cy, cx + half * 1.3, cy, 1.8, glow);
    draw_line(cx, cy - half * 1.3, cx, cy + half * 1.3, 1.8, glow);
}

/// Draw a crisp rightward play arrow
pub fn draw_vector_play(cx: f32, cy: f32, size: f32, color: Color) {
    let h = size * 0.55;
    let w = size * 0.48;
    let p1 = Vec2::new(cx - w * 0.5, cy - h);
    let p2 = Vec2::new(cx + w * 0.65, cy);
    let p3 = Vec2::new(cx - w * 0.5, cy + h);
    draw_triangle(p1, p2, p3, color);
}

/// Draw two pause bars
pub fn draw_vector_pause(cx: f32, cy: f32, width: f32, height: f32, color: Color) {
    let bar_w = width * 0.32;
    let gap = width * 0.36;
    draw_rectangle(cx - gap * 0.5 - bar_w, cy - height * 0.5, bar_w, height, color);
    draw_rectangle(cx + gap * 0.5, cy - height * 0.5, bar_w, height, color);
}

/// Draw a padlock icon for locked sectors
pub fn draw_vector_lock(cx: f32, cy: f32, size: f32, color: Color) {
    let body_w = size * 0.75;
    let body_h = size * 0.55;
    let body_y = cy - body_h * 0.15;
    let shackle_r = size * 0.28;

    // Shackle
    draw_circle_lines(cx, body_y, shackle_r, 2.5, color);

    // Body
    draw_rectangle(cx - body_w * 0.5, body_y, body_w, body_h, color);

    // Keyhole
    draw_circle(cx, body_y + body_h * 0.42, size * 0.08, Color::new(0.08, 0.08, 0.14, 1.0));
    draw_rectangle(cx - size * 0.04, body_y + body_h * 0.42, size * 0.08, size * 0.18, Color::new(0.08, 0.08, 0.14, 1.0));
}

/// Draw an X close mark
pub fn draw_vector_close(cx: f32, cy: f32, size: f32, thickness: f32, color: Color) {
    let half = size * 0.5;
    draw_line(cx - half, cy - half, cx + half, cy + half, thickness, color);
    draw_line(cx + half, cy - half, cx - half, cy + half, thickness, color);
}

/// Draw a procedural transparent gear / settings icon centered at (cx, cy)
pub fn draw_vector_gear(cx: f32, cy: f32, radius: f32, color: Color) {
    let teeth = 6;
    let step = PI / 3.0;
    let tooth_r = radius * 0.24;
    let hub_r = radius * 0.58;

    // 6 teeth protruding along the circumference
    for i in 0..teeth {
        let angle = i as f32 * step;
        let tx = cx + angle.cos() * (radius * 0.78);
        let ty = cy + angle.sin() * (radius * 0.78);
        draw_circle(tx, ty, tooth_r, color);
    }

    // Outer wheel ring (hollow center)
    draw_circle_lines(cx, cy, hub_r, radius * 0.30, color);
    // Center axle pip
    draw_circle(cx, cy, radius * 0.16, color);
}

