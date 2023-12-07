use bevy::prelude::*;
// use bevy::utils::Instant;

use crate::game::{settings::SettingsResource, states::GameState, world::WorldBoundaryResource};

use super::{
    components::{IconTransform, IconType, IconVelocity, Type},
    resources::{SpatialIndexResource, UpdateTimer},
    spatial::SpatialIndex,
};

type RoamingQuery = (
    Entity,
    &'static mut IconTransform,
    &'static mut IconVelocity,
    &'static IconType,
);

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

fn get_nearest_point_in_perimeter(point: Vec2, boundary_min: Vec2, boundary_max: Vec2) -> Vec2 {
    // Clamp the point within the boundary
    let clamped_point = point.clamp(boundary_min, boundary_max);

    // Calculate distances to the boundaries
    let dl = (clamped_point.x - boundary_min.x).abs();
    let dr = (clamped_point.x - boundary_max.x).abs();
    let dt = (clamped_point.y - boundary_min.y).abs();
    let db = (clamped_point.y - boundary_max.y).abs();

    // Find the minimum distance to determine the nearest edge
    let m = dl.min(dr.min(dt.min(db)));

    // Return the nearest point on the perimeter based on the nearest edge
    if m == dt {
        Vec2::new(clamped_point.x, boundary_min.y)
    } else if m == db {
        Vec2::new(clamped_point.x, boundary_max.y)
    } else if m == dl {
        Vec2::new(boundary_min.x, clamped_point.y)
    } else {
        Vec2::new(boundary_max.x, clamped_point.y)
    }
}

// Function to calculate the avoidance force
fn calculate_avoidance_force(
    position: Vec2,
    boundary_min: Vec2,
    boundary_max: Vec2,
    max_force: f32,
    some_threshold_distance: f32,
) -> Option<Vec2> {
    // find the closest point to the boundary
    let closest = get_nearest_point_in_perimeter(position, boundary_min, boundary_max);
    let distance = (closest - position).length();

    if distance < some_threshold_distance {
        // get the vector from position to closest:
        let d = closest - position;
        Some(d.normalize() * max_force * -1.0)
    } else {
        None
    }
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
    target_position: &Vec2,
    icon_type: &IconType,
    _query: &Query<RoamingQuery>,
) -> Vec2 {
    let mut max_force = settings.max_force;
    let mut max_speed = settings.max_speed;

    if icon_type.0 == Type::Captured {
        return Vec2::ZERO;
    }

    if icon_type.0 == Type::Follower {
        max_speed = settings.seek_max_speed;
        max_force = settings.seek_max_force;
    }

    let collision_force = get_separation_force(
        entity,
        *position,
        rotation,
        *velocity,
        spatial_index,
        settings.collision_distance,
        max_speed,
        max_force,
    );
    let separation_force = get_separation_force(
        entity,
        *position,
        rotation,
        *velocity,
        spatial_index,
        settings.collision_distance,
        max_speed,
        max_force,
    );
    let alignment_force = get_alignment_force(
        entity,
        *position,
        rotation,
        *velocity,
        spatial_index,
        settings.alignment_distance,
        max_speed,
        max_force,
    );
    let cohesion_force = get_cohesion_force(
        entity,
        *position,
        rotation,
        *velocity,
        spatial_index,
        settings.cohesion_distance,
        max_speed,
        max_force,
    );

    // println!("collision_force: {:?}", collision_force);
    // println!("separation_force: {:?}", separation_force);

    let mut acceleration = separation_force * settings.separation_weight;
    acceleration += alignment_force * settings.alignment_weight;
    acceleration += cohesion_force * settings.cohesion_weight;
    acceleration += collision_force * settings.collision_weight;

    if icon_type.0 == Type::Follower {
        let force = get_seek_force(*position, *velocity, *target_position, max_speed, max_force);
        acceleration += force * settings.seek_weight;
    }

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
    if let Some(force) = calculate_avoidance_force(
        *position,
        boundaries.dropzone_min,
        boundaries.dropzone_max,
        settings.avoidance_force_dropzone,
        settings.avoidance_distance_dropzone,
    ) {
        acceleration = force;
    }

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

    acceleration = limit_vec2(acceleration, max_force);

    // println!("velocity={:?} acceleration={:?}", velocity, acceleration);

    let mut velocity = limit_vec2(*velocity + acceleration, max_speed);

    if velocity.x.is_nan() {
        velocity.x = 0.0;
    }

    if velocity.y.is_nan() {
        velocity.y = 0.0;
    }

    velocity
}

fn update_icon_roaming_velocity(
    time: Res<Time>,
    mut timer: ResMut<UpdateTimer>,
    settings: Res<SettingsResource>,
    spatial_index: Res<SpatialIndexResource>,
    mut query: Query<RoamingQuery>,
    boundaries: Res<WorldBoundaryResource>,
) {
    timer.0.tick(time.delta());
    if !timer.0.finished() {
        return;
    }

    // let start = Instant::now();
    let player_position = query
        .iter()
        .find_map(|(_, IconTransform { position, .. }, _, icon_type)| {
            if icon_type.0 == Type::Player {
                Some(position)
            } else {
                None
            }
        })
        .unwrap();

    let velocities = query
        .iter()
        .map(
            |(entity, IconTransform { position, rotation }, velocity, icon_type)| {
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
                        player_position,
                        icon_type,
                        &query,
                    ),
                )
            },
        )
        .collect::<Vec<(Entity, Vec2)>>();

    for (entity, velocity) in velocities.into_iter() {
        if let Ok((_, mut icon_transform, mut icon_velocity, icon_type)) = query.get_mut(entity) {
            if icon_type.0 == Type::Player || icon_type.0 == Type::Captured {
                // player moves the icon
                continue;
            }

            icon_velocity.0 = velocity;

            // rotation/angle in radians from velocity vector...
            icon_transform.rotation = velocity.y.atan2(velocity.x);

            if icon_transform.rotation.is_nan() {
                println!("THIS IS A BUG rotation is NaN! velocity={:?}", velocity);
            }
        }
    }

    // debug!("update_icon_velocity in {:?}", start.elapsed());
}
