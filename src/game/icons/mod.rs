use bevy::prelude::*;
use bevy::render::batching::NoAutomaticBatching;
use bevy::render::mesh::shape;
use bevy::render::view::{NoFrustumCulling, RenderLayers};
use bevy::sprite::Mesh2dHandle;
use bevy::utils::Instant;
use rand::prelude::*;

use crate::game::camera::{CAMERA_LAYER, CAMERA_Z_ICONS};
use crate::game::icons::commands::CircleShapeCommand;
use crate::game::icons::components::{
    IconEntity, IconInstanceData, IconPlayerCircle, IconRenderEntity, IconVelocity, SheetIndex,
};
use crate::game::icons::resources::{HoveredIcon, SpatialIndexResource};

use self::resources::UpdateTimer;

use super::assets::icons::IconSheetAsset;
use super::settings::SettingsResource;
use super::states::GameState;
use super::world::WorldBoundaryResource;

mod capture;
pub mod commands;
mod components;
mod controller;
pub mod events;
pub mod health;
mod renderer;
mod resources;
mod roaming;
mod spatial;

pub use components::{IconPlayerController, IconSheetRef, IconTransform, IconType, Type};
pub use resources::IconSheetResource;

pub const ICON_SIZE: f32 = 32.0;
pub const ICON_CIRCLE_RADIUS: f32 = ICON_SIZE / 2.0 + 8.0;
pub const ICON_MIN_DISTANCE: f32 = 45.25 + 15.0;
pub const SPATIAL_GRID_SIZE: f32 = 128.0; // TODO: huge performance impact, tune this later!

pub struct IconPlugin;

impl Plugin for IconPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(HoveredIcon::default());
        app.insert_resource(UpdateTimer::default());
        app.add_plugins((
            renderer::IconRendererPlugin,
            roaming::IconRoamingPlugin,
            controller::IconPlayerControllerPlugin,
            capture::IconCapturePlugin,
            health::PlayerHealthPlugin,
        ));
        app.add_systems(OnEnter(GameState::MainMenu), init_icons_system);
        app.add_systems(
            Update,
            (
                apply_icon_velocity,
                update_icon_instance_data,
                fix_free_items_in_dropzone,
            )
                .chain()
                .run_if(in_state(GameState::GameRunning).or_else(in_state(GameState::MainMenu))),
        );
    }
}

fn random_position_in_bounds(rng: &mut ThreadRng, boundaries: &WorldBoundaryResource) -> Vec2 {
    loop {
        let position = Vec2::new(
            rng.gen_range(boundaries.bounds_min.x..boundaries.bounds_max.x),
            rng.gen_range(boundaries.bounds_min.y..boundaries.bounds_max.y),
        );
        if !boundaries.in_dropzone(position) {
            return position;
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn init_icons_system(
    mut commands: Commands,
    resource: Res<IconSheetResource>,
    assets: Res<Assets<IconSheetAsset>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut state: ResMut<NextState<GameState>>,
    boundaries: Res<WorldBoundaryResource>,

    player_circle: Query<Entity, With<IconPlayerCircle>>,
    existing_icons: Query<Entity, With<IconEntity>>,
    render_entity: Query<Entity, With<IconRenderEntity>>,

    settings: Res<SettingsResource>,
) {
    if let Ok(entity) = player_circle.get_single() {
        commands.entity(entity).despawn_recursive();
    }
    for entity in existing_icons.iter() {
        commands.entity(entity).despawn_recursive();
    }
    if let Ok(entity) = render_entity.get_single() {
        commands.entity(entity).despawn_recursive();
    }

    let WorldBoundaryResource {
        bounds_min,
        bounds_max,
        ..
    } = boundaries.as_ref();
    let IconSheetAsset(sheets) = assets.get(&resource.handle).unwrap();
    let mut rng = rand::thread_rng();
    let mut instances = Vec::new();
    let mut positions = Vec::new();
    let mut textures = Vec::new();
    let mut count = 0;

    let mut player_position = Vec2::ZERO;
    let mut player_ = false;

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
                    let mut position = random_position_in_bounds(&mut rng, &boundaries);

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

                    let mut rotation = (rng.gen_range(0.0..360.0) as f32).to_radians();

                    // let is_player = icon_index == 0 && !player_; //
                    let is_player = icon.name == "rust";
                    if is_player {
                        player_ = true;
                        position = Vec2::new(
                            0.0,
                            (boundaries.dropzone_max.y - boundaries.dropzone_min.y) * 0.25,
                        );
                        rotation = 0.0;
                        player_position = position;
                    }

                    if !is_player && (settings.max_icons != 0 && count > settings.max_icons) {
                        break;
                    }

                    let initial_speed = 1.0;
                    let velocity = if is_player {
                        Vec2::ZERO
                    } else {
                        Vec2::new(
                            rotation.cos() * initial_speed,
                            rotation.sin() * initial_speed,
                        )
                    };

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
                            IconType(if is_player { Type::Player } else { Type::Free }),
                        ))
                        .id();

                    // perhaps use the bevy icon instead?
                    if is_player {
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
    // info!("Fitted {} icons in {:?}", count, start.elapsed());

    // info!("Spawned {} icons", count);

    let mesh = Mesh::from(shape::Quad {
        size: Vec2::splat(ICON_SIZE),
        flip: false,
    });
    let mesh_handle = meshes.add(mesh);
    commands.spawn((
        IconRenderEntity,
        Mesh2dHandle(mesh_handle),
        SpatialBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, CAMERA_Z_ICONS)),
            ..Default::default()
        },
        RenderLayers::layer(CAMERA_LAYER),
        IconInstanceData::new(resource.texture_array.clone().unwrap(), instances),
        NoFrustumCulling,
        NoAutomaticBatching,
    ));

    commands.add(CircleShapeCommand::<IconPlayerCircle> {
        radius: ICON_CIRCLE_RADIUS,
        position: player_position,
        color: "#444c56",
        tag: IconPlayerCircle,
        ..Default::default()
    });

    commands.insert_resource(SpatialIndexResource(spatial_index));

    // state.set(GameState::MainMenu);
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
    mut query: Query<(Entity, &mut IconTransform, &mut IconVelocity, &IconType)>,
    mut player_circle: Query<&mut Transform, With<IconPlayerCircle>>,
    boundaries: Res<WorldBoundaryResource>,
) {
    for (entity, mut position, mut velocity, icon_type) in query.iter_mut() {
        if icon_type.0 == Type::Captured {
            continue;
        }

        let new_position =
            position.position + velocity.0 * (time.delta_seconds() * settings.velocity_time_scale);

        if !boundaries.in_bounds(new_position) {
            velocity.0 = -velocity.0;
            continue;
        }

        position.position = new_position;

        // update spatial index with new position and velocity
        spatial_index
            .0
            .insert(entity, position.position, velocity.0);

        if icon_type.0 == Type::Player {
            let mut player_circle = player_circle.single_mut();
            player_circle.translation.x = position.position.x;
            player_circle.translation.y = position.position.y;
        }
    }
}

// sometimes it might happen that free roaming items occur inside the dropzone
// we manually move them outside the dropzone:
pub fn fix_free_items_in_dropzone(
    boundaries: Res<WorldBoundaryResource>,
    mut spatial_index: ResMut<SpatialIndexResource>,
    mut query: Query<(Entity, &mut IconTransform, &IconVelocity, &IconType)>,
    time: Res<Time>,
    mut timer: Local<Option<Timer>>,
) {
    if timer.is_none() {
        *timer = Some(Timer::from_seconds(1.0, TimerMode::Repeating));
    }

    if let Some(timer) = timer.as_mut() {
        timer.tick(time.delta());
        if !timer.finished() {
            return;
        }
    }

    let mut rng = rand::thread_rng();
    for (entity, mut transform, velocity, icon_type) in query.iter_mut() {
        if icon_type.0 == Type::Free && boundaries.in_dropzone(transform.position) {
            let new_position = random_position_in_bounds(&mut rng, &boundaries);

            transform.position = new_position;
            spatial_index.0.insert(entity, new_position, velocity.0);
        }
    }
}
