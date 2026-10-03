use crate::physics::CelestialTier;

// ─────────────────────────────────────────────────────────────────────────────
//  Sector Objective
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum Objective {
    /// Earn at least this many points in the run.
    ReachScore(u64),
    /// Create at least one body of the specified tier (via merge).
    CreateTier(CelestialTier),
    /// Create the specified number of bodies of the given tier.
    CreateCount { tier: CelestialTier, count: u32 },
}

impl Objective {
    pub fn display_text(&self) -> String {
        match self {
            Objective::ReachScore(n) => format!("Score {}", n),
            Objective::CreateTier(t) => format!("Create a {}", t.name()),
            Objective::CreateCount { tier, count } => {
                format!("Create {} x {}", count, tier.name())
            }
        }
    }

    pub fn short_hint(&self) -> &'static str {
        match self {
            Objective::ReachScore(_) => "Merge bigger bodies for more points!",
            Objective::CreateTier(t) => match t {
                CelestialTier::Moon           => "Merge 2 Asteroids to make a Moon",
                CelestialTier::Terrestrial    => "Merge 2 Moons to make an Earth",
                CelestialTier::GasGiant       => "Merge 2 Earths to make a Gas Giant",
                CelestialTier::RingedGiant    => "Merge 2 Gas Giants to make Saturn",
                CelestialTier::IceGiant       => "Merge 2 Saturns to make an Ice Giant",
                CelestialTier::RedDwarf       => "Merge 2 Ice Giants to make a Red Dwarf",
                CelestialTier::BlueSupergiant => "Merge 2 Red Dwarfs for a Supergiant",
                CelestialTier::Pulsar         => "Merge 2 Supergiants for a Pulsar",
                CelestialTier::Singularity    => "The ultimate merge awaits!",
                _ => "Keep merging!",
            },
            Objective::CreateCount { .. } => "Keep dropping and merging!",
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Sector Definition
// ─────────────────────────────────────────────────────────────────────────────

pub struct Sector {
    pub name: &'static str,
    pub region: &'static str,
    pub objective: Objective,
    /// Flavour text shown on the galaxy map.
    pub lore: &'static str,
}

// ─────────────────────────────────────────────────────────────────────────────
//  Sector Catalogue  (3 chapters × 5 sectors = 15 total)
// ─────────────────────────────────────────────────────────────────────────────

pub fn all_sectors() -> Vec<Sector> {
    vec![
        // ── Chapter 1: Nebula Rim ─────────────────────────────────────────────
        Sector {
            name: "First Contact",
            region: "Nebula Rim",
            objective: Objective::ReachScore(200),
            lore: "A cloud of loose debris drifts at the galaxy's edge.",
        },
        Sector {
            name: "Moon Cluster",
            region: "Nebula Rim",
            objective: Objective::CreateTier(CelestialTier::Moon),
            lore: "Ancient moonlets circle a forgotten world.",
        },
        Sector {
            name: "Birthworld",
            region: "Nebula Rim",
            objective: Objective::CreateTier(CelestialTier::Terrestrial),
            lore: "A young rocky planet stirs with potential life.",
        },
        Sector {
            name: "Gas Rush",
            region: "Nebula Rim",
            objective: Objective::ReachScore(1500),
            lore: "A vast cloud of hydrogen collapses into giants.",
        },
        Sector {
            name: "Ring of Saturn",
            region: "Nebula Rim",
            objective: Objective::CreateTier(CelestialTier::RingedGiant),
            lore: "A majestic ringed world needs your help to form.",
        },

        // ── Chapter 2: Frost Expanse ──────────────────────────────────────────
        Sector {
            name: "Ice Fields",
            region: "Frost Expanse",
            objective: Objective::CreateTier(CelestialTier::IceGiant),
            lore: "Frozen giants lurk in the outer dark.",
        },
        Sector {
            name: "Twin Giants",
            region: "Frost Expanse",
            objective: Objective::CreateCount { tier: CelestialTier::RingedGiant, count: 2 },
            lore: "A double-ringed system awaits restoration.",
        },
        Sector {
            name: "Stellar Birth",
            region: "Frost Expanse",
            objective: Objective::ReachScore(6000),
            lore: "Energy ripples as the first stars ignite.",
        },
        Sector {
            name: "Red Giant",
            region: "Frost Expanse",
            objective: Objective::CreateTier(CelestialTier::RedDwarf),
            lore: "A dying sun swells to engulf its worlds.",
        },
        Sector {
            name: "Blue Supergiant",
            region: "Frost Expanse",
            objective: Objective::CreateTier(CelestialTier::BlueSupergiant),
            lore: "The hottest, most luminous star in the expanse.",
        },

        // ── Chapter 3: Dark Matter Void ───────────────────────────────────────
        Sector {
            name: "Pulsar Temple",
            region: "Dark Matter Void",
            objective: Objective::CreateTier(CelestialTier::Pulsar),
            lore: "A spinning neutron star broadcasts across the void.",
        },
        Sector {
            name: "Void Surge",
            region: "Dark Matter Void",
            objective: Objective::ReachScore(20000),
            lore: "Dark energy floods the sector. Keep merging.",
        },
        Sector {
            name: "Twin Pulsars",
            region: "Dark Matter Void",
            objective: Objective::CreateCount { tier: CelestialTier::Pulsar, count: 2 },
            lore: "Binary pulsars spiral toward collapse.",
        },
        Sector {
            name: "Event Horizon",
            region: "Dark Matter Void",
            objective: Objective::ReachScore(50000),
            lore: "Reality bends at the rim of the black hole.",
        },
        Sector {
            name: "Singularity",
            region: "Dark Matter Void",
            objective: Objective::CreateTier(CelestialTier::Singularity),
            lore: "Create the ultimate cosmic object. The galaxy is saved.",
        },
    ]
}

// ─────────────────────────────────────────────────────────────────────────────
//  Run-time Objective Tracker
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ObjectiveTracker {
    pub objective: Objective,
    pub count_progress: u32,   // for CreateCount objectives
    pub score_progress: u64,   // current score (updated externally)
}

impl ObjectiveTracker {
    pub fn new(objective: Objective) -> Self {
        Self { objective, count_progress: 0, score_progress: 0 }
    }

    /// Call when a new body is created via merge.
    pub fn on_merge(&mut self, new_tier: CelestialTier) {
        match &self.objective {
            Objective::CreateTier(t) => {
                if new_tier == *t {
                    self.count_progress += 1;
                }
            }
            Objective::CreateCount { tier, .. } => {
                if new_tier == *tier {
                    self.count_progress += 1;
                }
            }
            Objective::ReachScore(_) => {}
        }
    }

    /// Returns true when the objective has been met.
    pub fn is_complete(&self) -> bool {
        match &self.objective {
            Objective::ReachScore(target) => self.score_progress >= *target,
            Objective::CreateTier(_) => self.count_progress >= 1,
            Objective::CreateCount { count, .. } => self.count_progress >= *count,
        }
    }

    /// 0.0 – 1.0 progress fraction for the HUD progress bar.
    pub fn progress_fraction(&self) -> f32 {
        match &self.objective {
            Objective::ReachScore(target) => {
                (self.score_progress as f32 / *target as f32).min(1.0)
            }
            Objective::CreateTier(_) => {
                if self.count_progress >= 1 { 1.0 } else { 0.0 }
            }
            Objective::CreateCount { count, .. } => {
                (self.count_progress as f32 / *count as f32).min(1.0)
            }
        }
    }

    pub fn progress_text(&self) -> String {
        match &self.objective {
            Objective::ReachScore(target) => {
                format!("{} / {}", self.score_progress, target)
            }
            Objective::CreateTier(_) => {
                if self.count_progress >= 1 { "DONE!".to_string() } else { "Not yet".to_string() }
            }
            Objective::CreateCount { count, .. } => {
                format!("{} / {}", self.count_progress, count)
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Star-rating helper
// ─────────────────────────────────────────────────────────────────────────────

/// Rate a completed sector run. Currently: always 1 star for completing the
/// objective; future expansions can add time-based and score-based stars.
pub fn rate_run(_sector_idx: usize, _score: u64) -> u8 {
    1 // minimum for now — 2-3 star targets can be tuned later
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_sectors_count() {
        assert_eq!(all_sectors().len(), 15, "should have exactly 15 sectors");
    }

    #[test]
    fn test_score_objective() {
        let mut tracker = ObjectiveTracker::new(Objective::ReachScore(500));
        assert!(!tracker.is_complete());
        tracker.score_progress = 499;
        assert!(!tracker.is_complete());
        tracker.score_progress = 500;
        assert!(tracker.is_complete());
    }

    #[test]
    fn test_create_tier_objective() {
        let mut tracker = ObjectiveTracker::new(Objective::CreateTier(CelestialTier::Moon));
        assert!(!tracker.is_complete());
        tracker.on_merge(CelestialTier::Asteroid); // wrong tier
        assert!(!tracker.is_complete());
        tracker.on_merge(CelestialTier::Moon);
        assert!(tracker.is_complete());
    }

    #[test]
    fn test_create_count_objective() {
        let mut tracker = ObjectiveTracker::new(Objective::CreateCount {
            tier: CelestialTier::RingedGiant,
            count: 2,
        });
        tracker.on_merge(CelestialTier::RingedGiant);
        assert!(!tracker.is_complete());
        tracker.on_merge(CelestialTier::RingedGiant);
        assert!(tracker.is_complete());
    }
}
