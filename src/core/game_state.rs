#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    MainMenu,
    Playing,
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
