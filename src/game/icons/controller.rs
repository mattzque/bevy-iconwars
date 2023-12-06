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
        let turn = if keys.any_pressed([KeyCode::Left, KeyCode::Q]) {
            // turn left
            Some(1.0)
        } else if keys.any_pressed([KeyCode::Right, KeyCode::E]) {
            // turn right
            Some(-1.0)
        } else {
            None
        };

        if let Some(turn) = turn {
            transform.rotation += turn * dt * settings.controller_turn_speed;
            transform.rotation = normalize_angle(transform.rotation);
        }

        // get forward and strafe vectors from camera rotation
        let rotation = transform.rotation;
        // // vector pointing forwards relative to the camera rotation, ignoring the y axis
        // let forward_vector = {
        //     let f = rotation.mul_vec3(Vec3::Z).normalize();
        //     Vec3::new(f.x, 0.0, f.z).normalize()
        // };
        // // vector pointing left/right horizontally relative to the camera rotation
        // let strafe_vector = Quat::from_rotation_y(90.0f32.to_radians())
        //     .mul_vec3(forward_vector)
        //     .normalize();

        let r = std::f32::consts::PI / 2.0;
        // Vector pointing forwards relative to the camera rotation
        let forward_vector = Vec2::new((rotation - r).cos(), (rotation - r).sin());

        // Vector pointing left/right horizontally relative to the camera rotation
        // In 2D, this is a 90 degrees (or π/2 radians) rotation of the forward vector
        let strafe_vector = Vec2::new(-forward_vector.y, forward_vector.x);

        let mut accel = Vec2::ZERO;
        if keys.any_pressed([KeyCode::Up, KeyCode::W]) {
            // forward
            accel += forward_vector * -1.0;
        }
        if keys.any_pressed([KeyCode::Down, KeyCode::S]) {
            // backward
            accel += forward_vector * 1.0;
        }
        if keys.pressed(KeyCode::A) {
            // strafe left
            accel += strafe_vector * -1.0;
        }
        if keys.pressed(KeyCode::D) {
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

        velocity_.0 = velocity;
        // println!("updated velocity: {:?}", velocity);
    }
}
