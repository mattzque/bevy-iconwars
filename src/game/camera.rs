use bevy::prelude::*;

use super::states::GameState;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_cameras);
        app.add_systems(OnEnter(GameState::GameRunning), make_camera_visible);
    }
}

fn setup_cameras(mut commands: Commands) {
    commands.spawn((Camera2dBundle {
        projection: OrthographicProjection {
            scale: 1.7,
            ..Default::default()
        },
        camera: Camera {
            order: 1,
            is_active: true,
            ..Default::default()
        },
        ..Default::default()
    },));
}

fn make_camera_visible(mut query: Query<&mut Camera, With<Camera3d>>) {
    query.single_mut().is_active = true;
}
