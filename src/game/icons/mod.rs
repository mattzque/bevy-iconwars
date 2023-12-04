use std::time::Instant;

use bevy::prelude::*;
use bevy::render::mesh::shape;
use bevy::render::view::NoFrustumCulling;
use bevy::sprite::Mesh2dHandle;
use bevy::window::PrimaryWindow;
use rand::prelude::*;

use crate::game::icons::components::{
    IconEntity, IconInstanceData, IconRenderEntity, IconSheetRef, IconTransform, SheetIndex,
};
use crate::game::icons::resources::{HoveredIcon, SpatialIndexResource};

use super::assets::icons::IconSheetAsset;
use super::camera::CameraTag;
use super::states::GameState;

mod components;
mod renderer;
mod resources;
mod spatial;

pub use resources::IconSheetResource;

pub const ICON_SIZE: f32 = 32.0;
pub const ICON_MIN_DISTANCE: f32 = 45.25 + 15.0;
pub const SPATIAL_GRID_SIZE: f32 = 256.0;

pub struct IconPlugin;

impl Plugin for IconPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(HoveredIcon::default());
        app.add_plugins(renderer::IconRendererPlugin);
        app.add_systems(OnEnter(GameState::GameLoading), init_icons_system);
        app.add_systems(
            Update,
            (debug_icons_system, update_hovered_icon_system)
                .run_if(in_state(GameState::GameRunning)),
        );
    }
}

fn init_icons_system(
    mut commands: Commands,
    resource: Res<IconSheetResource>,
    assets: Res<Assets<IconSheetAsset>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut state: ResMut<NextState<GameState>>,
) {
    let (bounds_min, bounds_max) = (Vec2::new(-2000.0, -2000.0), Vec2::new(2000.0, 2000.0));
    let IconSheetAsset(sheets) = assets.get(&resource.handle).unwrap();
    let mut rng = rand::thread_rng();
    let mut positions = Vec::new();
    let mut textures = Vec::new();
    let mut indices = Vec::new();
    let mut count = 0;
    let start = Instant::now();
    sheets.iter().enumerate().for_each(|(sheet_index, sheet)| {
        textures.push(sheet.handle.clone());
        sheet
            .tiles
            .iter()
            .enumerate()
            .for_each(|(icon_index, icon)| {
                loop {
                    // candidate:
                    let position = Vec2::new(
                        rng.gen_range(bounds_min.x..bounds_max.x),
                        rng.gen_range(bounds_min.y..bounds_max.y),
                    );

                    // search for collisions:
                    let mut collision = false;
                    for (other_position, _, _) in &positions {
                        let d: Vec2 = position - *other_position;
                        if d.length() < ICON_MIN_DISTANCE {
                            collision = true;
                            break;
                        }
                    }
                    // try again if colliding with some other icon
                    if collision {
                        continue;
                    }

                    let rotation = (rng.gen_range(0.0..360.0) as f32).to_radians();
                    let components = (
                        IconEntity,
                        IconSheetRef {
                            sheet_index,
                            icon_index,
                            icon_name: icon.name.clone(),
                        },
                        IconTransform { position, rotation },
                    );
                    positions.push((position, rotation, components));
                    indices.push(SheetIndex {
                        sheet_index: sheet_index as u32,
                        tile_uv: Vec2::new(
                            icon.x as f32, // / sheet.width as f32,
                            icon.y as f32, // / sheet.height as f32,
                        ),
                    });
                    count += 1;
                    break;
                }
            });
    });
    info!("Fitted {} icons in {:?}", count, start.elapsed());

    let mut spatial_index = spatial::SpatialIndex::new(bounds_min, bounds_max, SPATIAL_GRID_SIZE);

    let transforms = positions
        .iter()
        .map(|(position, rotation, _)| Vec3::new(position.x, position.y, *rotation))
        .collect();
    positions.into_iter().for_each(|(position, _, components)| {
        let entity = commands.spawn(components).id();
        spatial_index.insert(position, entity);
    });

    info!("Spawned {} icons", count);

    let mesh = Mesh::from(shape::Quad {
        size: Vec2::splat(ICON_SIZE),
        flip: false,
    });
    let mesh_handle = meshes.add(mesh);
    commands.spawn((
        IconRenderEntity,
        Mesh2dHandle(mesh_handle),
        SpatialBundle {
            transform: Transform::from_translation(Vec3::ZERO),
            ..Default::default()
        },
        IconInstanceData::new(resource.texture_array.clone().unwrap(), transforms, indices),
        NoFrustumCulling,
    ));
    commands.insert_resource(SpatialIndexResource(spatial_index));

    state.set(GameState::GameRunning);
}

fn debug_icons_system(
    query: Query<(Entity, &IconTransform)>,
    hovered: Res<HoveredIcon>,
    mut gizmos: Gizmos,
) {
    for (entity, IconTransform { position, .. }) in query.iter() {
        // gizmos.rect_2d(*position, *rotation, Vec2::splat(ICON_SIZE), Color::RED);
        gizmos.circle_2d(
            *position,
            ICON_SIZE / 2.0 + 8.0,
            if hovered.0 == Some(entity) {
                Color::RED
            } else {
                Color::BLUE
            },
        );
    }
}

fn update_hovered_icon_system(
    mut commands: Commands,
    window: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform), With<CameraTag>>,
    index: Res<SpatialIndexResource>,
) {
    let (camera, camera_transform) = camera.single();
    let window = window.single();
    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
    {
        let maybe_entity = index.0.query(world_position, ICON_SIZE / 2.0 + 8.0).next();
        commands.insert_resource(HoveredIcon(maybe_entity));
    }
}
