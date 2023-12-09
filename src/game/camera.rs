use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use bevy::window::WindowResized;

use super::icons::{IconPlayerController, IconTransform};
use super::states::GameState;
use super::world::WorldBoundaryResource;

pub const CAMERA_LAYER: u8 = 1;
pub const CAMERA_LAYER_UI: u8 = 2;

pub const CAMERA_Z_BACKGROUND: f32 = -10.0;
pub const CAMERA_Z_ICONS: f32 = 0.0;
pub const CAMERA_Z_VFX: f32 = 1.0;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_cameras);
        app.add_systems(OnEnter(GameState::MainMenu), make_camera_visible);
        app.add_systems(OnEnter(GameState::GameRunning), enter_game_running_system);

        app.add_systems(
            Update,
            resize_menu_camera.run_if(in_state(GameState::MainMenu)),
        );

        app.add_systems(
            Update,
            (camera_zoom_system, camera_follow_player_icon_system)
                .run_if(in_state(GameState::GameRunning)),
        );
    }
}

#[derive(Component)]
pub struct CameraTag;

#[derive(Component)]
pub struct UiCameraTag;

fn setup_cameras(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                order: 0,
                is_active: false,
                ..Default::default()
            },
            ..Default::default()
        },
        RenderLayers::layer(CAMERA_LAYER),
        CameraTag,
    ));

    commands.spawn((
        Camera2dBundle {
            camera_2d: Camera2d {
                clear_color: ClearColorConfig::None, // ::Custom(Color::rgba(0.0, 0.0, 0.0, 0.0)),
            },
            camera: Camera {
                order: 1,
                is_active: true,
                ..Default::default()
            },
            ..Default::default()
        },
        RenderLayers::layer(CAMERA_LAYER_UI),
        UiCameraTag,
    ));
}

fn make_camera_visible(
    mut query: Query<(&mut Transform, &mut Camera, &mut OrthographicProjection), With<CameraTag>>,
    window: Query<&Window>,
    boundaries: Res<WorldBoundaryResource>,
) {
    let window = window.single();

    let world_size = boundaries.size();
    let max_scale = (world_size.x / window.width()).max(world_size.y / window.height());

    let (mut transform, mut camera, mut projection) = query.single_mut();

    camera.is_active = true;
    projection.scale = max_scale;
    transform.translation = Vec3::ZERO;
}

fn enter_game_running_system(mut query: Query<&mut OrthographicProjection, With<CameraTag>>) {
    let mut projection = query.single_mut();
    projection.scale = 1.0;
}

fn resize_menu_camera(
    mut query: Query<(&mut Camera, &mut OrthographicProjection), With<CameraTag>>,
    window: Query<&Window>,
    boundaries: Res<WorldBoundaryResource>,
    mut resize_reader: EventReader<WindowResized>,
) {
    for _ in resize_reader.read() {
        let window = window.single();

        let world_size = boundaries.size();
        let max_scale = (world_size.x / window.width()).max(world_size.y / window.height());

        let (mut camera, mut projection) = query.single_mut();

        camera.is_active = true;
        projection.scale = max_scale;
    }
}

fn camera_zoom_system(
    mut query: Query<&mut OrthographicProjection, With<Camera>>,
    window: Query<&Window>,
    mut scroll_events: EventReader<MouseWheel>,
    boundaries: Res<WorldBoundaryResource>,
) {
    for event in scroll_events.read() {
        for mut projection in query.iter_mut() {
            let window = window.single();

            const SCALE_FACTOR: f32 = 0.3;

            let event_y = if event.y > 0.0 {
                1.0
            } else if event.y < 0.0 {
                -1.0
            } else {
                0.0
            };

            projection.scale += (event_y * SCALE_FACTOR) * -1.0;

            let world_size = boundaries.size();
            let max_scale = (world_size.x / window.width()).max(world_size.y / window.height());

            if projection.scale < 1.0 {
                projection.scale = 1.0;
            }
            if projection.scale > max_scale {
                projection.scale = max_scale;
            }
        }
    }
}

fn camera_follow_player_icon_system(
    player_icon: Query<&IconTransform, With<IconPlayerController>>,
    mut query: Query<&mut Transform, With<CameraTag>>,
) {
    if let Ok(IconTransform { position, .. }) = player_icon.get_single() {
        let mut camera = query.single_mut();
        camera.translation.x = position.x;
        camera.translation.y = position.y;
    }
}
