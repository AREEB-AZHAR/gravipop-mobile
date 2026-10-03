/// Top-level game screens and states.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    MainMenu,
    GalaxyMap,          // browse / select sectors
    Playing,            // the actual drop game
    SectorComplete,     // objective met — celebrate + advance
    Paused,
    GameOver,
    Shop,
    WatchingAd,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdWatchReason {
    ReviveRun,
    DoubleStardust,
    FreeDailyChest,
    TestAd,
}
