use bevy::prelude::*;

use crate::game::{settings::SettingsResource, states::GameState};

use super::{
    components::{IconTransform, IconVelocity},
    IconPlayerController,
};

pub struct IconPlayerControllerPlugin;

impl Plugin for IconPlayerControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            update_key_input.run_if(in_state(GameState::GameRunning)),
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

fn update_key_input(
    time: Res<Time>,
    mut query: Query<(Entity, &mut IconTransform, &mut IconVelocity), With<IconPlayerController>>,
    keys: Res<Input<KeyCode>>,
    settings: Res<SettingsResource>,
) {
    let dt = time.delta_seconds();
    if let Ok((_entity, mut transform, mut velocity_)) = query.get_single_mut() {
        let turn = if keys.any_pressed([KeyCode::Left, KeyCode::A]) {
            // turn left
            Some(1.0)
        } else if keys.any_pressed([KeyCode::Right, KeyCode::D]) {
            // turn right
            Some(-1.0)
        } else {
            None
        };

        if let Some(turn) = turn {
            transform.rotation += turn * dt * settings.controller_turn_speed;
            transform.rotation = normalize_angle(transform.rotation);
        }

        let rotation = transform.rotation;

        let r = std::f32::consts::PI / 2.0;
        let forward_vector = Vec2::new((rotation - r).cos(), (rotation - r).sin());

        let mut accel = Vec2::ZERO;
        if keys.any_pressed([KeyCode::Up, KeyCode::W]) {
            // forward
            accel += forward_vector * -1.0;
        }
        if keys.any_pressed([KeyCode::Down, KeyCode::S]) {
            // backward
            accel += forward_vector * 1.0;
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

        velocity_.0 = velocity;
    }
}
