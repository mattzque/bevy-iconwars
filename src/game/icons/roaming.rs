use bevy::prelude::*;
use bevy::utils::Instant;

use crate::game::{settings::SettingsResource, states::GameState, world::WorldBoundaryResource};

use super::{
    components::{IconTransform, IconVelocity},
    resources::{SpatialIndexResource, UpdateTimer},
    spatial::SpatialIndex,
    IconPlayerController,
};

pub struct IconRoamingPlugin;

impl Plugin for IconRoamingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            update_icon_roaming_velocity.run_if(in_state(GameState::GameRunning)),
        );
    }
}

fn limit_vec2(vector: Vec2, max_length: f32) -> Vec2 {
    if vector.length() > max_length {
        vector.normalize() * max_length
    } else {
        vector
    }
}

/// Separation, steer away from nearby boids
///
/// Returns: separation force vector
#[allow(clippy::too_many_arguments)]
fn get_separation_force(
    entity: Entity,
    position: Vec2,
    _rotation: &f32,
    velocity: Vec2,
    spatial_index: &SpatialIndex<Entity>,
    separation_distance: f32,
    max_speed: f32,
    max_force: f32,
) -> Vec2 {
    let mut steer = Vec2::ZERO;
    let mut count = 0;

    for result in spatial_index.query(position, separation_distance) {
        if result.key == entity {
            continue;
        }
        let mut diff = position - *result.position;
        diff = diff.normalize();
        diff /= result.distance;
        steer += diff;
        count += 1;
    }
    if count > 0 {
        steer /= count as f32;
    }
    if steer.length() > 0.0 {
        steer = steer.normalize();
        steer *= max_speed;
        steer -= velocity;
        steer = limit_vec2(steer, max_force);
    }
    steer
}

/// Alignment, steer along with the average velocity of nearby boids
///
/// Returns: alignment force vector
#[allow(clippy::too_many_arguments)]
fn get_alignment_force(
    entity: Entity,
    position: Vec2,
    _rotation: &f32,
    velocity: Vec2,
    spatial_index: &SpatialIndex<Entity>,
    alignment_distance: f32,
    max_speed: f32,
    max_force: f32,
) -> Vec2 {
    let mut average_velocity = Vec2::ZERO;
    let mut count = 0;
    for result in spatial_index.query(position, alignment_distance) {
        if result.key == entity {
            continue;
        }
        average_velocity += *result.velocity;
        count += 1;
    }
    if count > 0 {
        average_velocity /= count as f32;
        average_velocity = average_velocity.normalize();
        average_velocity *= max_speed;
        average_velocity -= velocity;
        average_velocity = limit_vec2(average_velocity, max_force);
        average_velocity
    } else {
        Vec2::ZERO
    }
}

fn get_seek_force(
    position: Vec2,
    velocity: Vec2,
    target: Vec2,
    max_speed: f32,
    max_force: f32,
) -> Vec2 {
    let mut desired = target - position;
    if desired == Vec2::ZERO {
        return Vec2::ZERO;
    }
    desired = desired.normalize();
    desired *= max_speed;
    desired -= velocity;
    desired = limit_vec2(desired, max_force);
    desired
}

/// Cohesion, steer towards the average position of nearby boids
///
/// Returns: cohesion force vector
#[allow(clippy::too_many_arguments)]
fn get_cohesion_force(
    _entity: Entity,
    position: Vec2,
    _rotation: &f32,
    velocity: Vec2,
    spatial_index: &SpatialIndex<Entity>,
    cohesion_distance: f32,
    max_speed: f32,
    max_force: f32,
) -> Vec2 {
    let mut average_position = Vec2::ZERO;
    let mut count = 0;
    for result in spatial_index.query(position, cohesion_distance) {
        average_position += *result.position;
        count += 1;
    }
    if count > 0 {
        average_position /= count as f32;
        if average_position.length() > 0.0 {
            get_seek_force(position, velocity, average_position, max_speed, max_force)
        } else {
            Vec2::ZERO
        }
    } else {
        Vec2::ZERO
    }
}

type RoamingQuery = (
    Entity,
    &'static mut IconTransform,
    &'static mut IconVelocity,
);

// Function to calculate the avoidance force
fn calculate_avoidance_force(
    position: Vec2,
    boundary_min: Vec2,
    boundary_max: Vec2,
    max_force: f32,
    some_threshold_distance: f32,
) -> Vec2 {
    let mut force = Vec2::new(0.0, 0.0);

    // find the closest point to the boundary
    let closest_x = position.x.clamp(boundary_min.x, boundary_max.x);
    let closest_y = position.y.clamp(boundary_min.y, boundary_max.y);
    let distance = (Vec2::new(closest_x, closest_y) - position).length();

    if distance < some_threshold_distance {
        // calculate the force
        let mut force_x = 0.0;
        let mut force_y = 0.0;

        if closest_x == boundary_min.x {
            force_x = max_force * (1.0 - (position.x - boundary_min.x) / some_threshold_distance);
        } else if closest_x == boundary_max.x {
            force_x = max_force * (1.0 - (boundary_max.x - position.x) / some_threshold_distance);
        }

        if closest_y == boundary_min.y {
            force_y = max_force * (1.0 - (position.y - boundary_min.y) / some_threshold_distance);
        } else if closest_y == boundary_max.y {
            force_y = max_force * (1.0 - (boundary_max.y - position.y) / some_threshold_distance);
        }

        force = Vec2::new(force_x, force_y);
    }

    // // For the X-axis
    // let distance_to_min_x = (position.x - boundary_min.x).abs();
    // let distance_to_max_x = (boundary_max.x - position.x).abs();
    // let min_distance_x = distance_to_min_x.min(distance_to_max_x);
    // force.x = max_force * (1.0 - min_distance_x / some_threshold_distance).clamp(0.0, 1.0);
    // force.x *= if distance_to_min_x < distance_to_max_x {
    //     -1.0
    // } else {
    //     1.0
    // };

    // // For the Y-axis
    // let distance_to_min_y = (position.y - boundary_min.y).abs();
    // let distance_to_max_y = (boundary_max.y - position.y).abs();
    // let min_distance_y = distance_to_min_y.min(distance_to_max_y);
    // force.y = max_force * (1.0 - min_distance_y / some_threshold_distance).clamp(0.0, 1.0);
    // force.y *= if distance_to_min_y < distance_to_max_y {
    //     -1.0
    // } else {
    //     1.0
    // };

    force
}

#[allow(clippy::too_many_arguments)]
fn get_icon_velocity(
    entity: Entity,
    position: &Vec2,
    rotation: &f32,
    velocity: &Vec2,
    spatial_index: &SpatialIndex<Entity>,
    settings: &SettingsResource,
    boundaries: &WorldBoundaryResource,
) -> Vec2 {
    let collision_force = get_separation_force(
        entity,
        *position,
        rotation,
        *velocity,
        spatial_index,
        settings.collision_distance,
        settings.max_speed,
        settings.max_force,
    );
    let separation_force = get_separation_force(
        entity,
        *position,
        rotation,
        *velocity,
        spatial_index,
        settings.collision_distance,
        settings.max_speed,
        settings.max_force,
    );
    let alignment_force = get_alignment_force(
        entity,
        *position,
        rotation,
        *velocity,
        spatial_index,
        settings.alignment_distance,
        settings.max_speed,
        settings.max_force,
    );
    let cohesion_force = get_cohesion_force(
        entity,
        *position,
        rotation,
        *velocity,
        spatial_index,
        settings.cohesion_distance,
        settings.max_speed,
        settings.max_force,
    );

    // println!("collision_force: {:?}", collision_force);
    // println!("separation_force: {:?}", separation_force);

    let mut acceleration = separation_force * settings.separation_weight;
    acceleration += alignment_force * settings.alignment_weight;
    acceleration += cohesion_force * settings.cohesion_weight;
    acceleration += collision_force * settings.collision_weight;

    // if let Some(target_position) = target.position {
    //     let force = get_seek_force(
    //         position.0,
    //         velocity.0,
    //         target_position,
    //         settings.max_speed,
    //         settings.max_force,
    //     );
    //     acceleration += force * settings.seek_weight;
    // }

    // Boundary avoidance
    if position.x < boundaries.bounds_min.x {
        acceleration.x = settings.max_force;
    }
    if position.x > boundaries.bounds_max.x {
        acceleration.x = -settings.max_force;
    }
    if position.y < boundaries.bounds_min.y {
        acceleration.y = settings.max_force;
    }
    if position.y > boundaries.bounds_max.y {
        acceleration.y = -settings.max_force;
    }

    // Inside the main logic
    let force = calculate_avoidance_force(
        *position,
        boundaries.dropzone_min,
        boundaries.dropzone_max,
        settings.avoidance_force_dropzone,
        settings.avoidance_distance_dropzone,
    );
    acceleration.x += force.x;
    acceleration.y += force.y;

    // // Check against the inner drop zone boundaries
    // if position.x > boundaries.dropzone_min.x && position.x < boundaries.dropzone_max.x {
    //     if position.x - boundaries.dropzone_min.x < boundaries.dropzone_max.x - position.x {
    //         acceleration.x = -settings.max_force; // Push left
    //     } else {
    //         acceleration.x = settings.max_force; // Push right
    //     }
    // }
    // if position.y > boundaries.dropzone_min.y && position.y < boundaries.dropzone_max.y {
    //     if position.y - boundaries.dropzone_min.y < boundaries.dropzone_max.y - position.y {
    //         acceleration.y = -settings.max_force; // Push up
    //     } else {
    //         acceleration.y = settings.max_force; // Push down
    //     }
    // }

    acceleration = limit_vec2(acceleration, settings.max_force);

    // println!("velocity={:?} acceleration={:?}", velocity, acceleration);

    limit_vec2(*velocity + acceleration, settings.max_speed)
}

fn update_icon_roaming_velocity(
    time: Res<Time>,
    mut timer: ResMut<UpdateTimer>,
    settings: Res<SettingsResource>,
    spatial_index: Res<SpatialIndexResource>,
    mut query: Query<RoamingQuery, Without<IconPlayerController>>,
    boundaries: Res<WorldBoundaryResource>,
) {
    timer.0.tick(time.delta());
    if !timer.0.finished() {
        return;
    }

    let start = Instant::now();

    let velocities = query
        .iter()
        .map(|(entity, IconTransform { position, rotation }, velocity)| {
            (
                entity,
                get_icon_velocity(
                    entity,
                    position,
                    rotation,
                    &velocity.0,
                    &spatial_index.0,
                    &settings,
                    &boundaries,
                ),
            )
        })
        .collect::<Vec<(Entity, Vec2)>>();

    for (entity, velocity) in velocities.into_iter() {
        if let Ok((_, mut icon_transform, mut icon_velocity)) = query.get_mut(entity) {
            icon_velocity.0 = velocity;

            // rotation/angle in radians from velocity vector...
            icon_transform.rotation = velocity.y.atan2(velocity.x);
        }
    }

    debug!("update_icon_velocity in {:?}", start.elapsed());
}
