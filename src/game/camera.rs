use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

use super::icons::{IconPlayerController, IconTransform};
use super::states::GameState;

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

fn setup_cameras(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle {
            // projection: OrthographicProjection {
            //     // scale: 2.7,
            //     // scale: 0.8,
            //     ..Default::default()
            // },
            // camera: Camera {
            //     // order: 1,
            //     // is_active: true,
            //     ..Default::default()
            // },
            ..Default::default()
        },
        CameraTag,
    ));
}

fn make_camera_visible(mut query: Query<&mut Camera, With<Camera>>) {
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
    mut query: Query<&mut Transform, With<Camera>>,
) {
    if let Ok(IconTransform { position, .. }) = player_icon.get_single() {
        let mut camera = query.single_mut();
        camera.translation.x = position.x;
        camera.translation.y = position.y;
    }
}
