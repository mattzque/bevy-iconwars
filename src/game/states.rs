use bevy::prelude::*;

#[derive(States, Clone, Copy, Default, Eq, PartialEq, Debug, Hash)]
pub enum GameState {
    /// Schedule asset loading
    #[default]
    Init,
    /// Assets are loading
    AssetsLoading,
    /// All Assets are loaded
    AssetsLoaded,
    /// Game is initializing
    GameLoading,

    MainMenu,

    /// Game is running
    GameRunning,

    /// Player died or won
    GameOver,
}
