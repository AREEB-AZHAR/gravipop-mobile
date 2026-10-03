#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RewardType {
    EventHorizonRevive,
    DoubleStardust,
    DailyChest,
    DebugAd,
}

#[allow(dead_code)]
pub trait AdService {
    fn is_rewarded_ready(&self, reward: RewardType) -> bool;
    fn start_rewarded_ad(&mut self, reward: RewardType);
    fn start_interstitial_ad(&mut self);
    fn is_ad_playing(&self) -> bool;
    fn get_ad_time_remaining(&self) -> f32;
    fn update(&mut self, dt: f32) -> Option<RewardType>;
    fn skip_or_finish(&mut self) -> Option<RewardType>;
}

pub struct MockAdService {
    active_reward: Option<RewardType>,
    timer: f32,
    duration: f32,
    is_interstitial: bool,
}

impl MockAdService {
    pub fn new() -> Self {
        Self {
            active_reward: None,
            timer: 0.0,
            duration: 5.0, // 5-second simulated video ad
            is_interstitial: false,
        }
    }
}

impl AdService for MockAdService {
    fn is_rewarded_ready(&self, _reward: RewardType) -> bool {
        true
    }

    fn start_rewarded_ad(&mut self, reward: RewardType) {
        self.active_reward = Some(reward);
        self.timer = self.duration;
        self.is_interstitial = false;
    }

    fn start_interstitial_ad(&mut self) {
        self.active_reward = Some(RewardType::DebugAd);
        self.timer = 3.0;
        self.is_interstitial = true;
    }

    fn is_ad_playing(&self) -> bool {
        self.active_reward.is_some()
    }

    fn get_ad_time_remaining(&self) -> f32 {
        self.timer
    }

    fn update(&mut self, dt: f32) -> Option<RewardType> {
        if self.active_reward.is_some() {
            self.timer -= dt;
            if self.timer <= 0.0 {
                let reward = self.active_reward.take();
                return reward;
            }
        }
        None
    }

    fn skip_or_finish(&mut self) -> Option<RewardType> {
        let reward = self.active_reward.take();
        self.timer = 0.0;
        reward
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_ad_reward_cycle() {
        let mut service = MockAdService::new();
        assert!(!service.is_ad_playing());

        service.start_rewarded_ad(RewardType::EventHorizonRevive);
        assert!(service.is_ad_playing());
        assert_eq!(service.get_ad_time_remaining(), 5.0);

        // Advance 2 seconds
        let reward = service.update(2.0);
        assert_eq!(reward, None);
        assert!(service.is_ad_playing());

        // Advance remaining 3.5 seconds
        let reward = service.update(3.5);
        assert_eq!(reward, Some(RewardType::EventHorizonRevive));
        assert!(!service.is_ad_playing());
    }

    #[test]
    fn test_mock_ad_skip_claim() {
        let mut service = MockAdService::new();
        service.start_rewarded_ad(RewardType::DoubleStardust);
        let reward = service.skip_or_finish();
        assert_eq!(reward, Some(RewardType::DoubleStardust));
        assert!(!service.is_ad_playing());
    }
}
