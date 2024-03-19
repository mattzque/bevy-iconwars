use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::game::{camera::CameraTag, settings::SettingsResource, states::GameState};

use super::{
    components::{IconTransform, IconVelocity},
    IconPlayerController,
};

pub struct IconPlayerControllerPlugin;

impl Plugin for IconPlayerControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (update_key_input, update_player_rotation).run_if(in_state(GameState::GameRunning)),
        );
    }
}

fn normalize_angle(mut angle: f32) -> f32 {
    let two_pi = 2.0 * std::f32::consts::PI;
    angle %= two_pi;
    if angle < 0.0 {
        angle += two_pi;
    }
    angle
}

// player rotation by mouse position
fn update_player_rotation(
    window: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform), With<CameraTag>>,
    mut query: Query<&mut IconTransform, With<IconPlayerController>>,
) {
    let (camera, camera_transform) = camera.single();
    let window = window.single();
    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
    {
        if let Ok(mut transform) = query.get_single_mut() {
            let delta = world_position - transform.position.trunc();
            let rotation = delta.y.atan2(delta.x);
            let r = std::f32::consts::PI / 2.0;
            let rotation = normalize_angle(rotation - r);
            transform.rotation = rotation;
        }
    }
}

fn update_key_input(
    time: Res<Time>,
    mut query: Query<(Entity, &IconTransform, &mut IconVelocity), With<IconPlayerController>>,
    keys: Res<ButtonInput<KeyCode>>,
    settings: Res<SettingsResource>,
) {
    let dt = time.delta_seconds();
    // info!("print dt = {:?}", dt);
    if let Ok((_entity, transform, mut velocity_)) = query.get_single_mut() {
        // let turn = if keys.any_pressed([KeyCode::KeyQ]) {
        //     // turn left
        //     Some(1.0)
        // } else if keys.any_pressed([KeyCode::KeyE]) {
        //     // turn right
        //     Some(-1.0)
        // } else {
        //     None
        // };

        // if let Some(turn) = turn {
        //     transform.rotation += turn * dt * settings.controller_turn_speed;
        //     transform.rotation = normalize_angle(transform.rotation);
        // }

        // let rotation = transform.rotation;
        let rotation = 0.0; // transform.rotation;

        let r = std::f32::consts::PI / 2.0;
        let forward_vector = Vec2::new((rotation - r).cos(), (rotation - r).sin());
        let strafe_vector = Vec2::new(rotation.cos(), rotation.sin());

        let mut accel = Vec2::ZERO;
        if keys.any_pressed([KeyCode::ArrowUp, KeyCode::KeyW]) {
            // forward
            accel += forward_vector * -1.0;
        }
        if keys.any_pressed([KeyCode::ArrowDown, KeyCode::KeyS]) {
            // backward
            accel += forward_vector * 1.0;
        }
        if keys.any_pressed([KeyCode::ArrowLeft, KeyCode::KeyA]) {
            // strafe left
            accel += strafe_vector * -1.0;
        }
        if keys.any_pressed([KeyCode::ArrowRight, KeyCode::KeyD]) {
            // strafe right
            accel += strafe_vector * 1.0;
        }

        // normalized and scaled by acceleration setting
        let accel: Vec2 = if accel.length() > 0.0 {
            accel.normalize() * (settings.controller_acceleration * dt)
        } else {
            Vec2::ZERO
        };

        let mut velocity = velocity_.0;

        // reduce / dampen velocity
        let friction: Vec2 = if velocity.length() != 0.0 {
            velocity.normalize() * -1.0 * (settings.controller_dampening * dt)
        } else {
            Vec2::ZERO
        };

        // apply acceleration to velocity /2???
        velocity += accel;

        // clamp velocity
        if velocity.length() > (settings.controller_max_speed * dt) {
            velocity = velocity.normalize() * (settings.controller_max_speed * dt);
        }

        // apply friction
        let delta_friction = friction * dt;
        // clamp friction not to go negative
        velocity = if (velocity + delta_friction).signum() != velocity.signum() {
            Vec2::ZERO
        } else {
            velocity + delta_friction
        };
        // if velocity.length() > 0.0 {
        //     info!(
        //         "dt={:?} velocity={:?} l = {:?}",
        //         dt,
        //         velocity,
        //         velocity.length()
        //     );
        // }

        velocity_.0 = velocity;
    }
}
