use super::CelestialTier;
use quad_rand::gen_range;

/// Per-run progress is advanced exclusively by confirmed physics merge events.
/// Unlockable drops stay at least two tiers below the highest merged planet.
#[derive(Default)]
pub struct DropPool {
    merges: u32,
    unlocked_at: [Option<u32>; 13],
}

impl DropPool {
    pub fn record_merge(&mut self, formed: CelestialTier) -> Option<CelestialTier> {
        self.merges = self.merges.saturating_add(1);
        let mut newest = None;
        for index in 3..CelestialTier::ALL.len() {
            // First unlock: merging two Saturns produces Ice Giant (index 5),
            // which adds only Gas Giant (index 3) to the three basic drops.
            if index + 2 <= formed as usize && self.unlocked_at[index].is_none() {
                self.unlocked_at[index] = Some(self.merges);
                newest = Some(CelestialTier::ALL[index]);
            }
        }
        newest
    }

    fn weights(&self) -> [f32; 13] {
        let mut weights = [0.0; 13];
        weights[..3].copy_from_slice(&[60.0, 30.0, 10.0]);
        for (index, weight) in weights.iter_mut().enumerate().skip(3) {
            if let Some(unlocked) = self.unlocked_at[index] {
                let progress = (self.merges.saturating_sub(unlocked) as f32 / 100.0).min(1.0);
                // Gas Giant starts at about 0.1%, reaching at most 0.6% after
                // 100 further merges. Each bigger tier is rarer still. The
                // combined probability of every unlocked large drop stays <2%.
                *weight = (0.1 + 0.5 * progress) / (index - 2) as f32;
            }
        }
        weights
    }

    pub fn random_tier(&self) -> CelestialTier {
        let weights = self.weights();
        let mut roll = gen_range(0.0, weights.iter().sum::<f32>());
        for (tier, weight) in CelestialTier::ALL.iter().zip(weights) {
            if roll < weight {
                return *tier;
            }
            roll -= weight;
        }
        CelestialTier::Asteroid
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_three_basics_are_allowed_until_merging_past_saturn() {
        let mut pool = DropPool::default();
        for _ in 0..1000 {
            pool.record_merge(CelestialTier::Terrestrial);
        }
        pool.record_merge(CelestialTier::RingedGiant);
        assert_eq!(pool.weights()[..3], [60.0, 30.0, 10.0]);
        assert!(pool.weights()[3..].iter().all(|weight| *weight == 0.0));
        assert_eq!(
            pool.record_merge(CelestialTier::IceGiant),
            Some(CelestialTier::GasGiant)
        );
        assert!(pool.weights()[3] > 0.0);
        assert!(pool.weights()[4..].iter().all(|weight| *weight == 0.0));
    }

    #[test]
    fn each_new_merge_milestone_adds_exactly_one_lower_tier() {
        let mut pool = DropPool::default();
        for formed in CelestialTier::ALL.iter().skip(5) {
            let index = *formed as usize;
            assert_eq!(
                pool.record_merge(*formed),
                Some(CelestialTier::ALL[index - 2])
            );
            assert!(pool.weights()[index - 2] > 0.0);
            assert!(pool.weights()[index - 1..]
                .iter()
                .all(|weight| *weight == 0.0));
            assert_eq!(pool.record_merge(*formed), None);
        }
    }

    #[test]
    fn rare_drop_probability_ramps_with_merges_and_stays_below_two_percent() {
        let mut pool = DropPool::default();
        pool.record_merge(CelestialTier::IceGiant);
        let initial = pool.weights()[3];
        for _ in 0..50 {
            pool.record_merge(CelestialTier::Moon);
        }
        let middle = pool.weights()[3];
        for _ in 0..50 {
            pool.record_merge(CelestialTier::Moon);
        }
        let maximum = pool.weights()[3];
        assert!(initial < middle && middle < maximum);
        assert!((initial - 0.1).abs() < 0.00001);
        assert!((maximum - 0.6).abs() < 0.00001);
        pool.record_merge(CelestialTier::CosmicCore);
        for _ in 0..1000 {
            pool.record_merge(CelestialTier::Moon);
        }
        let weights = pool.weights();
        assert!(weights[3..].iter().sum::<f32>() / weights.iter().sum::<f32>() < 0.02);
        assert_eq!(
            DropPool::default().weights()[3],
            0.0,
            "a new run resets every unlock"
        );
    }
}
