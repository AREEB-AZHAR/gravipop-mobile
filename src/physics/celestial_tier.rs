use macroquad::color::Color;

/// Cosmic tiers sized to fit in the 520 px jar.
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
    Nebula = 10,
    Quasar = 11,
    CosmicCore = 12,
}

impl CelestialTier {
    pub const ALL: [CelestialTier; 13] = [
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
        CelestialTier::Nebula,
        CelestialTier::Quasar,
        CelestialTier::CosmicCore,
    ];

    pub fn next_tier(&self) -> Option<CelestialTier> {
        let i = *self as usize;
        if i + 1 < Self::ALL.len() {
            Some(Self::ALL[i + 1])
        } else {
            None
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Asteroid => "Asteroid",
            Self::Moon => "Moon",
            Self::Terrestrial => "Earth",
            Self::GasGiant => "Gas Giant",
            Self::RingedGiant => "Saturn",
            Self::IceGiant => "Ice Giant",
            Self::RedDwarf => "Red Dwarf",
            Self::BlueSupergiant => "Supergiant",
            Self::Pulsar => "Pulsar",
            Self::Singularity => "SINGULARITY",
            Self::Nebula => "Nebula",
            Self::Quasar => "Quasar",
            Self::CosmicCore => "Cosmic Core",
        }
    }

    /// Radius in virtual pixels — deliberately airy so a full pot still has room to breathe.
    pub fn radius(&self) -> f32 {
        match self {
            Self::Asteroid => 18.0,
            Self::Moon => 25.0,
            Self::Terrestrial => 33.0,
            Self::GasGiant => 43.0,
            Self::RingedGiant => 54.0,
            Self::IceGiant => 67.0,
            Self::RedDwarf => 82.0,
            Self::BlueSupergiant => 98.0,
            Self::Pulsar => 116.0,
            Self::Singularity => 136.0,
            Self::Nebula => 150.0,
            Self::Quasar => 166.0,
            Self::CosmicCore => 182.0,
        }
    }

    /// Mass used only for collision impulse calculations.
    pub fn mass(&self) -> f32 {
        let r = self.radius();
        r * r * 0.008 // proportional to area, tuned for a soft floaty stack
    }

    /// Points awarded when this tier is produced by a merge.
    pub fn score_value(&self) -> u64 {
        match self {
            Self::Asteroid => 5,
            Self::Moon => 15,
            Self::Terrestrial => 40,
            Self::GasGiant => 100,
            Self::RingedGiant => 250,
            Self::IceGiant => 600,
            Self::RedDwarf => 1_500,
            Self::BlueSupergiant => 4_000,
            Self::Pulsar => 10_000,
            Self::Singularity => 30_000, // winning moment!
            Self::Nebula => 60_000,
            Self::Quasar => 120_000,
            Self::CosmicCore => 250_000,
        }
    }

    pub fn primary_color(&self) -> Color {
        match self {
            Self::Asteroid => Color::new(0.65, 0.62, 0.58, 1.0), // grey rock
            Self::Moon => Color::new(0.72, 0.88, 0.96, 1.0),     // pale blue-white
            Self::Terrestrial => Color::new(0.22, 0.76, 0.52, 1.0), // blue-green
            Self::GasGiant => Color::new(0.96, 0.58, 0.22, 1.0), // orange
            Self::RingedGiant => Color::new(0.98, 0.82, 0.28, 1.0), // golden
            Self::IceGiant => Color::new(0.22, 0.62, 0.98, 1.0), // electric blue
            Self::RedDwarf => Color::new(0.96, 0.24, 0.28, 1.0), // red
            Self::BlueSupergiant => Color::new(0.28, 0.88, 1.00, 1.0), // bright cyan
            Self::Pulsar => Color::new(0.88, 0.38, 1.00, 1.0),   // vivid purple
            Self::Singularity => Color::new(0.08, 0.06, 0.16, 1.0), // near-black w/ glow
            Self::Nebula => Color::new(0.72, 0.25, 0.65, 1.0),
            Self::Quasar => Color::new(0.20, 0.85, 0.85, 1.0),
            Self::CosmicCore => Color::new(1.0, 0.70, 0.20, 1.0),
        }
    }

    pub fn glow_color(&self) -> Color {
        let c = self.primary_color();
        Color::new(c.r, c.g, c.b, 0.55)
    }

    /// Label colour for the tier name drawn on large bodies.
    pub fn label_color(&self) -> Color {
        match self {
            Self::Singularity => Color::new(0.85, 0.35, 1.0, 1.0),
            _ => Color::new(1.0, 1.0, 1.0, 0.90),
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tier_progression() {
        assert_eq!(
            CelestialTier::Asteroid.next_tier(),
            Some(CelestialTier::Moon)
        );
        assert_eq!(CelestialTier::Singularity.next_tier(), Some(CelestialTier::Nebula));
        assert_eq!(CelestialTier::CosmicCore.next_tier(), None);
    }

    #[test]
    fn test_sizes_increase() {
        for i in 0..CelestialTier::ALL.len() - 1 {
            let a = CelestialTier::ALL[i];
            let b = CelestialTier::ALL[i + 1];
            assert!(
                b.radius() > a.radius(),
                "{:?} should be larger than {:?}",
                b,
                a
            );
            assert!(b.score_value() > a.score_value());
        }
    }

    #[test]
    fn test_singularity_fits_jar() {
        // Singularity diameter must be less than the jar width
        use crate::core::config::JAR_WIDTH;
        for tier in CelestialTier::ALL {
            assert!(tier.radius() * 2.0 < JAR_WIDTH);
        }
    }

}
