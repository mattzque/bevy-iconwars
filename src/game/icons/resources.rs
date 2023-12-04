use bevy::prelude::*;

use crate::game::assets::icons::IconSheetAsset;

use super::spatial::SpatialIndex;

#[derive(Resource, Debug)]
pub struct IconSheetResource {
    pub handle: Handle<IconSheetAsset>,
    pub texture_array: Option<Handle<Image>>,
}

#[derive(Resource)]
pub struct SpatialIndexResource(pub SpatialIndex<Entity>);

#[derive(Resource, Default)]
pub struct HoveredIcon(pub Option<Entity>);
