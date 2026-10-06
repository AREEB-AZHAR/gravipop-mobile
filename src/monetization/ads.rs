use super::ad_bridge;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RewardType {
    EventHorizonRevive,
    DoubleStardust,
    DailyChest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdOutcome {
    RewardEarned(RewardType),
    Closed,
    RewardSkipped,
    Unavailable,
}

#[derive(Clone, Copy)]
struct AdRequest {
    id: u32,
    reward: Option<RewardType>,
}

/// Rewards come only from the SDK's earned-reward callback, after the ad closes.
/// Request IDs ensure delayed callbacks cannot reward a different run or ad.
#[derive(Default)]
pub struct PlatformAdService {
    serial: u32,
    active: Option<AdRequest>,
}

impl PlatformAdService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_rewarded_ready(&self) -> bool {
        self.active.is_none() && ad_bridge::is_ready(true)
    }

    pub fn start_rewarded_ad(&mut self, reward: RewardType) -> bool {
        self.start(Some(reward))
    }

    pub fn start_interstitial_ad(&mut self) -> bool {
        self.start(None)
    }

    fn start(&mut self, reward: Option<RewardType>) -> bool {
        if self.active.is_some() || !ad_bridge::is_ready(reward.is_some()) {
            return false;
        }
        self.serial = self.serial.wrapping_add(1).max(1);
        if !ad_bridge::show(self.serial, reward.is_some()) {
            return false;
        }
        self.active = Some(AdRequest {
            id: self.serial,
            reward,
        });
        true
    }

    pub fn update(&mut self) -> Option<AdOutcome> {
        let request = self.active?;
        self.finish(request.id, ad_bridge::poll(request.id))
    }

    fn finish(&mut self, request_id: u32, result: i32) -> Option<AdOutcome> {
        let request = self.active?;
        if result == 0 || request_id != request.id {
            return None;
        }
        self.active = None;
        Some(match result {
            1 => request
                .reward
                .map_or(AdOutcome::Closed, AdOutcome::RewardEarned),
            2 if request.reward.is_some() => AdOutcome::RewardSkipped,
            2 => AdOutcome::Closed,
            _ => AdOutcome::Unavailable,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pending(reward: Option<RewardType>) -> PlatformAdService {
        PlatformAdService {
            serial: 7,
            active: Some(AdRequest { id: 7, reward }),
        }
    }

    #[test]
    fn confirmed_reward_keeps_the_requested_reward_type_and_is_consumed_once() {
        for reward in [
            RewardType::EventHorizonRevive,
            RewardType::DoubleStardust,
            RewardType::DailyChest,
        ] {
            let mut ads = pending(Some(reward));
            assert_eq!(ads.finish(7, 1), Some(AdOutcome::RewardEarned(reward)));
            assert_eq!(ads.finish(7, 1), None);
        }
    }

    #[test]
    fn closing_early_or_failing_never_awards_a_reward() {
        assert_eq!(
            pending(Some(RewardType::DoubleStardust)).finish(7, 2),
            Some(AdOutcome::RewardSkipped)
        );
        assert_eq!(
            pending(Some(RewardType::EventHorizonRevive)).finish(7, 3),
            Some(AdOutcome::Unavailable)
        );
    }

    #[test]
    fn interstitial_completion_never_awards_a_gameplay_reward() {
        assert_eq!(pending(None).finish(7, 1), Some(AdOutcome::Closed));
    }

    #[test]
    fn stale_callbacks_and_waiting_cannot_complete_the_current_request() {
        let mut ads = pending(Some(RewardType::DoubleStardust));
        assert_eq!(ads.finish(6, 1), None);
        assert_eq!(ads.finish(7, 0), None);
        assert_eq!(
            ads.finish(7, 1),
            Some(AdOutcome::RewardEarned(RewardType::DoubleStardust))
        );
    }

    #[test]
    fn unsupported_desktop_ads_fail_without_simulating_a_reward() {
        let mut ads = PlatformAdService::new();
        assert!(!ads.start_rewarded_ad(RewardType::DoubleStardust));
        assert_eq!(ads.update(), None);
    }
}
