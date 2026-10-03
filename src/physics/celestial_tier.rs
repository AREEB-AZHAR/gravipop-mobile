use macroquad::color::Color;
use quad_rand::gen_range;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CelestialTier {
    Asteroid = 0,
    Moon = 1,
    Terrestrial = 2,
    GasGiant = 3,
    RingedGiant = 4,
    IceGiant = 5,
    RedDwarf = 6,
    BlueSupergiant = 7,
    Pulsar = 8,
    Singularity = 9,
}

impl CelestialTier {
    pub const ALL: [CelestialTier; 10] = [
        CelestialTier::Asteroid,
        CelestialTier::Moon,
        CelestialTier::Terrestrial,
        CelestialTier::GasGiant,
        CelestialTier::RingedGiant,
        CelestialTier::IceGiant,
        CelestialTier::RedDwarf,
        CelestialTier::BlueSupergiant,
        CelestialTier::Pulsar,
        CelestialTier::Singularity,
    ];

    pub fn next_tier(&self) -> Option<CelestialTier> {
        let index = *self as usize;
        if index + 1 < Self::ALL.len() {
            Some(Self::ALL[index + 1])
        } else {
            None
        }
    }

    #[allow(dead_code)]
    pub fn name(&self) -> &'static str {
        match self {
            CelestialTier::Asteroid => "Asteroid",
            CelestialTier::Moon => "Moon",
            CelestialTier::Terrestrial => "Terrestrial",
            CelestialTier::GasGiant => "Gas Giant",
            CelestialTier::RingedGiant => "Ringed Giant",
            CelestialTier::IceGiant => "Ice Giant",
            CelestialTier::RedDwarf => "Red Dwarf",
            CelestialTier::BlueSupergiant => "Blue Supergiant",
            CelestialTier::Pulsar => "Pulsar",
            CelestialTier::Singularity => "Cosmic Singularity",
        }
    }

    pub fn radius(&self) -> f32 {
        match self {
            CelestialTier::Asteroid => 18.0,
            CelestialTier::Moon => 24.0,
            CelestialTier::Terrestrial => 31.0,
            CelestialTier::GasGiant => 39.0,
            CelestialTier::RingedGiant => 47.0,
            CelestialTier::IceGiant => 55.0,
            CelestialTier::RedDwarf => 64.0,
            CelestialTier::BlueSupergiant => 74.0,
            CelestialTier::Pulsar => 84.0,
            CelestialTier::Singularity => 95.0,
        }
    }

    pub fn mass(&self) -> f32 {
        match self {
            CelestialTier::Asteroid => 1.0,
            CelestialTier::Moon => 2.2,
            CelestialTier::Terrestrial => 4.5,
            CelestialTier::GasGiant => 9.0,
            CelestialTier::RingedGiant => 18.0,
            CelestialTier::IceGiant => 35.0,
            CelestialTier::RedDwarf => 70.0,
            CelestialTier::BlueSupergiant => 140.0,
            CelestialTier::Pulsar => 280.0,
            CelestialTier::Singularity => 600.0,
        }
    }

    pub fn score_value(&self) -> u64 {
        match self {
            CelestialTier::Asteroid => 10,
            CelestialTier::Moon => 25,
            CelestialTier::Terrestrial => 60,
            CelestialTier::GasGiant => 150,
            CelestialTier::RingedGiant => 350,
            CelestialTier::IceGiant => 800,
            CelestialTier::RedDwarf => 1800,
            CelestialTier::BlueSupergiant => 4200,
            CelestialTier::Pulsar => 9500,
            CelestialTier::Singularity => 25000,
        }
    }

    pub fn primary_color(&self) -> Color {
        match self {
            CelestialTier::Asteroid => Color::new(0.68, 0.70, 0.74, 1.0),
            CelestialTier::Moon => Color::new(0.70, 0.88, 0.95, 1.0),
            CelestialTier::Terrestrial => Color::new(0.20, 0.75, 0.50, 1.0),
            CelestialTier::GasGiant => Color::new(0.96, 0.55, 0.20, 1.0),
            CelestialTier::RingedGiant => Color::new(0.98, 0.80, 0.25, 1.0),
            CelestialTier::IceGiant => Color::new(0.20, 0.60, 0.98, 1.0),
            CelestialTier::RedDwarf => Color::new(0.95, 0.22, 0.28, 1.0),
            CelestialTier::BlueSupergiant => Color::new(0.25, 0.85, 1.0, 1.0),
            CelestialTier::Pulsar => Color::new(0.85, 0.35, 1.0, 1.0),
            CelestialTier::Singularity => Color::new(0.10, 0.08, 0.18, 1.0),
        }
    }

    pub fn glow_color(&self) -> Color {
        match self {
            CelestialTier::Asteroid => Color::new(0.68, 0.70, 0.74, 0.3),
            CelestialTier::Moon => Color::new(0.70, 0.88, 0.95, 0.4),
            CelestialTier::Terrestrial => Color::new(0.20, 0.75, 0.50, 0.45),
            CelestialTier::GasGiant => Color::new(0.96, 0.55, 0.20, 0.5),
            CelestialTier::RingedGiant => Color::new(0.98, 0.80, 0.25, 0.55),
            CelestialTier::IceGiant => Color::new(0.20, 0.60, 0.98, 0.55),
            CelestialTier::RedDwarf => Color::new(0.95, 0.22, 0.28, 0.6),
            CelestialTier::BlueSupergiant => Color::new(0.25, 0.85, 1.0, 0.65),
            CelestialTier::Pulsar => Color::new(0.85, 0.35, 1.0, 0.7),
            CelestialTier::Singularity => Color::new(0.90, 0.40, 1.0, 0.75),
        }
    }

    pub fn random_spawn_tier() -> Self {
        let roll = gen_range(0, 100);
        if roll < 55 {
            CelestialTier::Asteroid
        } else if roll < 85 {
            CelestialTier::Moon
        } else {
            CelestialTier::Terrestrial
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tier_progression() {
        assert_eq!(CelestialTier::Asteroid.next_tier(), Some(CelestialTier::Moon));
        assert_eq!(CelestialTier::Moon.next_tier(), Some(CelestialTier::Terrestrial));
        assert_eq!(CelestialTier::Pulsar.next_tier(), Some(CelestialTier::Singularity));
        assert_eq!(CelestialTier::Singularity.next_tier(), None);
    }

    #[test]
    fn test_tier_physics_scaling() {
        for i in 0..CelestialTier::ALL.len() - 1 {
            let current = CelestialTier::ALL[i];
            let next = CelestialTier::ALL[i + 1];
            assert!(next.radius() > current.radius());
            assert!(next.mass() > current.mass());
            assert!(next.score_value() > current.score_value());
        }
    }
}
