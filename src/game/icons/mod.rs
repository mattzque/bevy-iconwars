use bevy::prelude::*;
use bevy::render::mesh::shape;
use bevy::render::view::NoFrustumCulling;
use bevy::sprite::Mesh2dHandle;
use bevy::utils::Instant;
use bevy::window::PrimaryWindow;
use rand::prelude::*;

use crate::game::icons::components::{
    IconEntity, IconInstanceData, IconRenderEntity, IconSheetRef, IconVelocity, SheetIndex,
};
use crate::game::icons::resources::{HoveredIcon, SpatialIndexResource, WorldBoundaryResource};

use self::resources::UpdateTimer;

use super::assets::icons::IconSheetAsset;
use super::camera::CameraTag;
use super::settings::SettingsResource;
use super::states::GameState;

mod components;
mod renderer;
mod resources;
mod roaming;
mod spatial;

pub use components::{IconPlayerController, IconTransform};
pub use resources::IconSheetResource;

pub const ICON_SIZE: f32 = 32.0;
pub const ICON_MIN_DISTANCE: f32 = 45.25 + 15.0;
pub const SPATIAL_GRID_SIZE: f32 = 128.0; // TODO: huge performance impact, tune this later!

pub struct IconPlugin;

impl Plugin for IconPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(HoveredIcon::default());
        app.insert_resource(UpdateTimer::default());
        app.insert_resource(WorldBoundaryResource::default());
        app.add_plugins((renderer::IconRendererPlugin, roaming::IconRoamingPlugin));
        app.add_systems(OnEnter(GameState::GameLoading), init_icons_system);
        app.add_systems(
            Update,
            (
                debug_icons_system,
                update_hovered_icon_system,
                apply_icon_velocity,
                update_icon_instance_data,
            )
                .chain()
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
    boundaries: Res<WorldBoundaryResource>,
) {
    let WorldBoundaryResource {
        bounds_min,
        bounds_max,
    } = boundaries.as_ref();
    let IconSheetAsset(sheets) = assets.get(&resource.handle).unwrap();
    let mut rng = rand::thread_rng();
    let mut instances = Vec::new();
    let mut positions = Vec::new();
    let mut textures = Vec::new();
    let mut count = 0;

    let mut spatial_index = spatial::SpatialIndex::new(*bounds_min, *bounds_max, SPATIAL_GRID_SIZE);

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
                    for other_position in &positions {
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
                    let velocity = Vec2::new(rotation.cos() * 1.0, rotation.sin() * 1.0);

                    let entity = commands
                        .spawn((
                            IconEntity,
                            IconSheetRef {
                                sheet_index,
                                icon_index,
                                icon_name: icon.name.clone(),
                            },
                            IconTransform { position, rotation },
                            IconVelocity(velocity),
                        ))
                        .id();

                    // perhaps use the bevy icon instead?
                    if icon.name == "rust" {
                        commands.entity(entity).insert(IconPlayerController);
                    }

                    spatial_index.insert(entity, position, velocity);

                    let sheet_index = SheetIndex {
                        sheet_index: sheet_index as u32,
                        tile_uv: Vec2::new(
                            icon.x as f32, // / sheet.width as f32,
                            icon.y as f32, // / sheet.height as f32,
                        ),
                    };

                    positions.push(position);

                    instances.push((
                        entity,
                        (Vec3::new(position.x, position.y, rotation), sheet_index),
                    ));

                    count += 1;
                    break;
                }
            });
    });
    info!("Fitted {} icons in {:?}", count, start.elapsed());

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
        IconInstanceData::new(resource.texture_array.clone().unwrap(), instances),
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
    for (entity, IconTransform { position, rotation }) in query.iter() {
        gizmos.rect_2d(
            *position,
            *rotation,
            Vec2::splat(ICON_SIZE),
            if hovered.0 == Some(entity) {
                Color::RED
            } else {
                Color::BLACK
            },
        );
        // gizmos.circle_2d(
        //     *position,
        //     ICON_SIZE / 2.0 + 8.0,
        // );
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
        let maybe_entity = index
            .0
            .query(world_position, ICON_SIZE / 2.0 + 8.0)
            .next()
            .map(|result| result.key);
        commands.insert_resource(HoveredIcon(maybe_entity));
    }
}

fn update_icon_instance_data(
    query: Query<(Entity, &IconTransform)>,
    mut instance_data: Query<&mut IconInstanceData>,
) {
    let mut instance_data = instance_data.get_single_mut().unwrap();
    for (entity, IconTransform { position, rotation }) in &query {
        instance_data.update_transform(entity, Vec3::new(position.x, position.y, *rotation));
    }
}

pub fn apply_icon_velocity(
    time: Res<Time>,
    settings: Res<SettingsResource>,
    mut spatial_index: ResMut<SpatialIndexResource>,
    mut query: Query<(Entity, &mut IconTransform, &IconVelocity)>,
) {
    for (entity, mut position, velocity) in query.iter_mut() {
        position.position += velocity.0 * (time.delta_seconds() * settings.velocity_time_scale);

        // rotation/angle in radians from velocity vector...
        position.rotation = velocity.0.y.atan2(velocity.0.x);

        // update spatial index with new position and velocity
        spatial_index
            .0
            .insert(entity, position.position, velocity.0);
    }
}
