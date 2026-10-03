// Orbital gravity engine is not used in the drop game.
// Kept as a stub so the module tree compiles cleanly.
use macroquad::prelude::Vec2;
use super::body::CelestialBody;

pub struct GravityEngine;

impl GravityEngine {
    pub fn new(_center: Vec2) -> Self { Self }
    #[allow(dead_code)]
    pub fn apply_forces(&self, _bodies: &mut [CelestialBody], _dt: f32) {}
}
