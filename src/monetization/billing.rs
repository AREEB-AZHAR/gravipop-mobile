use crate::core::config::{CELESTIAL_PASS_PRODUCT_ID, REMOVE_ADS_PRODUCT_ID};

#[allow(dead_code)]
pub trait BillingService {
    fn purchase_product(&mut self, product_id: &str) -> Result<String, String>;
    fn is_ad_removed(&self) -> bool;
    fn is_celestial_pass_unlocked(&self) -> bool;
    fn set_ad_removed(&mut self, removed: bool);
    fn set_celestial_pass(&mut self, unlocked: bool);
}

pub struct MockBillingService {
    pub ads_removed: bool,
    pub celestial_pass_unlocked: bool,
}

impl MockBillingService {
    pub fn new(ads_removed: bool, celestial_pass_unlocked: bool) -> Self {
        Self {
            ads_removed,
            celestial_pass_unlocked,
        }
    }
}

impl BillingService for MockBillingService {
    fn purchase_product(&mut self, product_id: &str) -> Result<String, String> {
        match product_id {
            REMOVE_ADS_PRODUCT_ID => {
                self.ads_removed = true;
                Ok("Ads permanently removed! Thank you for your support!".to_string())
            }
            CELESTIAL_PASS_PRODUCT_ID => {
                self.celestial_pass_unlocked = true;
                Ok("Celestial Star Pass activated! All premium cosmic skins unlocked!".to_string())
            }
            "com.gravipop.stardust.500" => Ok("500 Stardust credited!".to_string()),
            "com.gravipop.stardust.2000" => Ok("2,000 Stardust credited!".to_string()),
            "com.gravipop.stardust.10000" => Ok("10,000 Stardust credited!".to_string()),
            _ => Err("Unknown product SKU".to_string()),
        }
    }

    fn is_ad_removed(&self) -> bool {
        self.ads_removed
    }

    fn is_celestial_pass_unlocked(&self) -> bool {
        self.celestial_pass_unlocked
    }

    fn set_ad_removed(&mut self, removed: bool) {
        self.ads_removed = removed;
    }

    fn set_celestial_pass(&mut self, unlocked: bool) {
        self.celestial_pass_unlocked = unlocked;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_ads_purchase() {
        let mut billing = MockBillingService::new(false, false);
        assert!(!billing.is_ad_removed());

        let res = billing.purchase_product(REMOVE_ADS_PRODUCT_ID);
        assert!(res.is_ok());
        assert!(billing.is_ad_removed());
    }

    #[test]
    fn test_star_pass_purchase() {
        let mut billing = MockBillingService::new(false, false);
        assert!(!billing.is_celestial_pass_unlocked());

        let res = billing.purchase_product(CELESTIAL_PASS_PRODUCT_ID);
        assert!(res.is_ok());
        assert!(billing.is_celestial_pass_unlocked());
    }
}
