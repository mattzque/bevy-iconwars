use bevy::core_pipeline::clear_color::ClearColorConfig;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::render::camera::CameraOutputMode;
use bevy::render::render_resource::{BlendState, LoadOp};
use bevy::render::view::RenderLayers;

use super::icons::{IconPlayerController, IconTransform};
use super::states::GameState;

pub const CAMERA_LAYER: u8 = 1;
pub const CAMERA_LAYER_UI: u8 = 2;

pub const CAMERA_Z_BACKGROUND: f32 = -10.0;
pub const CAMERA_Z_ICONS: f32 = 0.0;
pub const CAMERA_Z_VFX: f32 = 1.0;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_cameras);
        app.add_systems(OnEnter(GameState::GameRunning), make_camera_visible);
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
                clear_color: ClearColorConfig::Custom(Color::rgba(0.0, 0.0, 0.0, 0.0)),
            },
            camera: Camera {
                order: 1,
                is_active: true,
                output_mode: CameraOutputMode::Write {
                    blend_state: Some(BlendState::ALPHA_BLENDING),
                    color_attachment_load_op: LoadOp::Load,
                },
                ..Default::default()
            },
            ..Default::default()
        },
        RenderLayers::layer(CAMERA_LAYER_UI),
        UiCameraTag,
    ));
}

fn make_camera_visible(mut query: Query<&mut Camera, With<CameraTag>>) {
    query.single_mut().is_active = true;
}

fn camera_zoom_system(
    mut query: Query<&mut OrthographicProjection, With<Camera>>,
    mut scroll_events: EventReader<MouseWheel>,
) {
    for event in scroll_events.read() {
        for mut projection in query.iter_mut() {
            #[cfg(not(target_arch = "wasm32"))]
            const SCALE_FACTOR: f32 = 10.0;
            #[cfg(target_arch = "wasm32")]
            const SCALE_FACTOR: f32 = 1000.0;

            projection.scale *= (1.0 + event.y / SCALE_FACTOR) * -1.0;
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
