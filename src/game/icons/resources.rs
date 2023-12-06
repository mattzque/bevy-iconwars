use std::time::Duration;

use bevy::prelude::*;

use crate::game::assets::icons::IconSheetAsset;

use super::spatial::SpatialIndex;

#[derive(Resource)]
pub struct UpdateTimer(pub Timer);

impl Default for UpdateTimer {
    fn default() -> Self {
        Self(Timer::new(Duration::from_millis(100), TimerMode::Repeating))
    }
}

#[derive(Resource, Debug)]
pub struct IconSheetResource {
    pub handle: Handle<IconSheetAsset>,
    pub texture_array: Option<Handle<Image>>,
}

#[derive(Resource)]
pub struct SpatialIndexResource(pub SpatialIndex<Entity>);

#[derive(Resource, Default)]
pub struct HoveredIcon(pub Option<Entity>);

#[derive(Resource)]
pub struct WorldBoundaryResource {
    pub bounds_min: Vec2,
    pub bounds_max: Vec2,
}

impl Default for WorldBoundaryResource {
    fn default() -> Self {
        Self {
            bounds_min: Vec2::new(-5000.0, -5000.0),
            bounds_max: Vec2::new(5000.0, 5000.0),
        }
    }
}
