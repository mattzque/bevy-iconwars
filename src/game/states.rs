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
    /// Game is running
    GameRunning,
}
