use bevy::prelude::*;
use bevy::utils::Instant;

use crate::game::{settings::SettingsResource, states::GameState};

use super::{
    components::{IconTransform, IconVelocity},
    resources::{SpatialIndexResource, UpdateTimer, WorldBoundaryResource},
    spatial::SpatialIndex,
    IconPlayerController,
};

pub struct IconPlayerControllerPlugin;

impl Plugin for IconPlayerControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            update_key_input.run_if(in_state(GameState::GameRunning)),
        );
    }
}

fn update_key_input(query: Query<(Entity, &IconTransform), With<IconPlayerController>>) {}
