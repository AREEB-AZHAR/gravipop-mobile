use crate::core::config::{CELESTIAL_PASS_PRODUCT_ID, REMOVE_ADS_PRODUCT_ID};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ShopItem {
    pub id: String,
    pub title: String,
    pub description: String,
    pub price_display: String,
    pub is_real_money: bool,
    pub stardust_price: u64,
}

pub struct EconomyCatalog {
    pub iap_items: Vec<ShopItem>,
    pub skin_items: Vec<ShopItem>,
}

impl EconomyCatalog {
    pub fn new() -> Self {
        let iap_items = vec![
            ShopItem {
                id: REMOVE_ADS_PRODUCT_ID.to_string(),
                title: "Remove Ads".to_string(),
                description: "Eliminates all interstitial ads permanently + 1.5x Stardust multiplier".to_string(),
                price_display: "$2.99".to_string(),
                is_real_money: true,
                stardust_price: 0,
            },
            ShopItem {
                id: CELESTIAL_PASS_PRODUCT_ID.to_string(),
                title: "Celestial Star Pass".to_string(),
                description: "Unlocks 30 tiers of premium celestial skins, trails, and custom themes".to_string(),
                price_display: "$4.99".to_string(),
                is_real_money: true,
                stardust_price: 0,
            },
            ShopItem {
                id: "com.gravipop.stardust.500".to_string(),
                title: "Stardust Pouch".to_string(),
                description: "Instant injection of 500 Cosmic Stardust".to_string(),
                price_display: "$0.99".to_string(),
                is_real_money: true,
                stardust_price: 0,
            },
            ShopItem {
                id: "com.gravipop.stardust.2000".to_string(),
                title: "Stardust Constellation".to_string(),
                description: "2,000 Cosmic Stardust (Best Value!)".to_string(),
                price_display: "$2.99".to_string(),
                is_real_money: true,
                stardust_price: 0,
            },
        ];

        let skin_items = vec![
            ShopItem {
                id: "Cosmic Neon".to_string(),
                title: "Cosmic Neon".to_string(),
                description: "Default radiant planetary glow palette".to_string(),
                price_display: "OWNED".to_string(),
                is_real_money: false,
                stardust_price: 0,
            },
            ShopItem {
                id: "Cyber Synth".to_string(),
                title: "Cyber Synth".to_string(),
                description: "Electrifying magenta and cyan cyber-grid body textures".to_string(),
                price_display: "300 Stardust".to_string(),
                is_real_money: false,
                stardust_price: 300,
            },
            ShopItem {
                id: "Deep Obsidian".to_string(),
                title: "Deep Obsidian".to_string(),
                description: "Dark crystalline facets with iridescent golden highlights".to_string(),
                price_display: "800 Stardust".to_string(),
                is_real_money: false,
                stardust_price: 800,
            },
            ShopItem {
                id: "Prismatic Astral".to_string(),
                title: "Prismatic Astral".to_string(),
                description: "Rainbow chromatic dispersion on every orbital body".to_string(),
                price_display: "2,000 Stardust".to_string(),
                is_real_money: false,
                stardust_price: 2000,
            },
        ];

        Self {
            iap_items,
            skin_items,
        }
    }
}
