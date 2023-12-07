use bevy::prelude::*;

use super::{settings::SettingsResource, states::GameState};

#[derive(Resource)]
pub struct FontResource {
    pub handle: Handle<Font>,
}

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::GameLoading), init_hud);
    }
}

pub fn init_hud(mut _settings: ResMut<SettingsResource>) {}
