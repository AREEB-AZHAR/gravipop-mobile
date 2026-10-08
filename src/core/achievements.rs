use crate::physics::celestial_tier::CelestialTier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AchievementCategory {
    Score,
    Merge,
}

#[derive(Debug, Clone)]
pub struct Achievement {
    pub id: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub category: AchievementCategory,
    pub icon: &'static str,
    pub target_value: u64,
}

pub const ALL_ACHIEVEMENTS: &[Achievement] = &[
    // ── Score Milestones (up to 1,000,000) ──────────────────────────────────
    Achievement {
        id: "score_10k",
        title: "Orbital Initiate",
        description: "Reach a score of 10,000 points",
        category: AchievementCategory::Score,
        icon: "✨",
        target_value: 10_000,
    },
    Achievement {
        id: "score_25k",
        title: "Stellar Apprentice",
        description: "Reach a score of 25,000 points",
        category: AchievementCategory::Score,
        icon: "🌠",
        target_value: 25_000,
    },
    Achievement {
        id: "score_50k",
        title: "Superflare Dynamo",
        description: "Reach 50,000 points & unlock Super Solar Flare",
        category: AchievementCategory::Score,
        icon: "☀️",
        target_value: 50_000,
    },
    Achievement {
        id: "score_100k",
        title: "Solar Master",
        description: "Reach a score of 100,000 points",
        category: AchievementCategory::Score,
        icon: "🔥",
        target_value: 100_000,
    },
    Achievement {
        id: "score_250k",
        title: "Galactic Architect",
        description: "Reach a score of 250,000 points",
        category: AchievementCategory::Score,
        icon: "🌌",
        target_value: 250_000,
    },
    Achievement {
        id: "score_500k",
        title: "Cosmic Voyager",
        description: "Reach a score of 500,000 points",
        category: AchievementCategory::Score,
        icon: "🛸",
        target_value: 500_000,
    },
    Achievement {
        id: "score_750k",
        title: "Universal Sovereign",
        description: "Reach a score of 750,000 points",
        category: AchievementCategory::Score,
        icon: "👑",
        target_value: 750_000,
    },
    Achievement {
        id: "score_1m",
        title: "1,000,000 COSMIC LEGEND",
        description: "Surpass 1,000,000 score in an epic endless run",
        category: AchievementCategory::Score,
        icon: "🏆",
        target_value: 1_000_000,
    },

    // ── Celestial Body Merges (All Tiers) ────────────────────────────────────
    Achievement {
        id: "merge_moon",
        title: "Lunar Cradle",
        description: "Successfully merge 2 Asteroids into a Moon",
        category: AchievementCategory::Merge,
        icon: "🌑",
        target_value: CelestialTier::Moon as u64,
    },
    Achievement {
        id: "merge_earth",
        title: "Terraformed World",
        description: "Successfully merge 2 Moons into an Earth",
        category: AchievementCategory::Merge,
        icon: "🌍",
        target_value: CelestialTier::Terrestrial as u64,
    },
    Achievement {
        id: "merge_gas_giant",
        title: "Great Red Spot",
        description: "Successfully merge into a Gas Giant",
        category: AchievementCategory::Merge,
        icon: "🪐",
        target_value: CelestialTier::GasGiant as u64,
    },
    Achievement {
        id: "merge_ringed_giant",
        title: "Crown of Saturn",
        description: "Successfully merge into a Ringed Giant",
        category: AchievementCategory::Merge,
        icon: "💫",
        target_value: CelestialTier::RingedGiant as u64,
    },
    Achievement {
        id: "merge_ice_giant",
        title: "Glacial Corona",
        description: "Successfully merge into an Ice Giant",
        category: AchievementCategory::Merge,
        icon: "💎",
        target_value: CelestialTier::IceGiant as u64,
    },
    Achievement {
        id: "merge_red_dwarf",
        title: "Stellar Ignition",
        description: "Successfully merge into a Red Dwarf star",
        category: AchievementCategory::Merge,
        icon: "🔴",
        target_value: CelestialTier::RedDwarf as u64,
    },
    Achievement {
        id: "merge_blue_supergiant",
        title: "Cyan Supernova",
        description: "Successfully merge into a Blue Supergiant",
        category: AchievementCategory::Merge,
        icon: "🔷",
        target_value: CelestialTier::BlueSupergiant as u64,
    },
    Achievement {
        id: "merge_pulsar",
        title: "Relativistic Jet",
        description: "Successfully merge into a Pulsar / Magnetar",
        category: AchievementCategory::Merge,
        icon: "⚡",
        target_value: CelestialTier::Pulsar as u64,
    },
    Achievement {
        id: "merge_singularity",
        title: "Event Horizon",
        description: "Successfully merge into a Singularity Black Hole",
        category: AchievementCategory::Merge,
        icon: "🕳️",
        target_value: CelestialTier::Singularity as u64,
    },
    Achievement {
        id: "merge_nebula",
        title: "Stellar Nursery",
        description: "Successfully merge into a glowing Nebula",
        category: AchievementCategory::Merge,
        icon: "🌸",
        target_value: CelestialTier::Nebula as u64,
    },
    Achievement {
        id: "merge_quasar",
        title: "Relic of Creation",
        description: "Successfully merge into a brilliant Quasar",
        category: AchievementCategory::Merge,
        icon: "💠",
        target_value: CelestialTier::Quasar as u64,
    },
    Achievement {
        id: "merge_cosmic_core",
        title: "Apex Cosmic Core",
        description: "Successfully merge two Quasars into the Apex Cosmic Core",
        category: AchievementCategory::Merge,
        icon: "⭐",
        target_value: CelestialTier::CosmicCore as u64,
    },
    Achievement {
        id: "merge_all_complete",
        title: "Omni-Fusion Pantheon",
        description: "Merge every single celestial body in the known universe",
        category: AchievementCategory::Merge,
        icon: "🎖️",
        target_value: 13,
    },
];

/// Checks current game status against achievements.
/// Returns a list of newly unlocked achievements.
pub fn check_achievements(
    score: u64,
    merged_tiers: &[usize],
    unlocked_ids: &mut Vec<String>,
) -> Vec<&'static Achievement> {
    let mut newly_unlocked = Vec::new();

    for ach in ALL_ACHIEVEMENTS {
        if unlocked_ids.iter().any(|id| id == ach.id) {
            continue;
        }

        let is_unlocked = match ach.category {
            AchievementCategory::Score => score >= ach.target_value,
            AchievementCategory::Merge => {
                if ach.id == "merge_all_complete" {
                    // Check if all tiers from 1 (Moon) through 12 (CosmicCore) have been merged
                    (1..=12).all(|tier_idx| merged_tiers.contains(&tier_idx))
                } else {
                    merged_tiers.contains(&(ach.target_value as usize))
                }
            }
        };

        if is_unlocked {
            unlocked_ids.push(ach.id.to_string());
            newly_unlocked.push(ach);
        }
    }

    newly_unlocked
}
