use bevy::prelude::*;
use bevy::utils::Instant;

use crate::game::{settings::SettingsResource, states::GameState};

use super::{
    components::{IconTransform, IconVelocity},
    resources::{SpatialIndexResource, UpdateTimer, WorldBoundaryResource},
    spatial::SpatialIndex,
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
    _query: &Query<RoamingQuery>,
    separation_distance: f32,
    max_speed: f32,
    max_force: f32,
) -> Vec2 {
    let mut steer = Vec2::ZERO;
    let mut count = 0;

    for (other_entity, other_position, distance) in
        spatial_index.query(position, separation_distance)
    {
        if other_entity == entity {
            continue;
        }
        let mut diff = position - other_position;
        diff = diff.normalize();
        diff /= distance;
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
    query: &Query<RoamingQuery>,
    alignment_distance: f32,
    max_speed: f32,
    max_force: f32,
) -> Vec2 {
    let mut average_velocity = Vec2::ZERO;
    let mut count = 0;
    for (other_entity, _, _) in spatial_index.query(position, alignment_distance) {
        if other_entity == entity {
            continue;
        }
        let (_, _, IconVelocity(other_velocity)) = query.get(other_entity).unwrap();
        average_velocity += *other_velocity;
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
    query: &Query<RoamingQuery>,
    cohesion_distance: f32,
    max_speed: f32,
    max_force: f32,
) -> Vec2 {
    let mut average_position = Vec2::ZERO;
    let mut count = 0;
    for (other_entity, _, _) in spatial_index.query(position, cohesion_distance) {
        let (
            _,
            IconTransform {
                position: other_position,
                ..
            },
            _,
        ) = query.get(other_entity).unwrap();
        average_position += *other_position;
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

type RoamingQuery = (Entity, &'static IconTransform, &'static mut IconVelocity);

#[allow(clippy::too_many_arguments)]
fn get_icon_velocity(
    entity: Entity,
    position: &Vec2,
    rotation: &f32,
    velocity: &Vec2,
    spatial_index: &SpatialIndex<Entity>,
    query: &Query<RoamingQuery>,
    settings: &SettingsResource,
    boundaries: &WorldBoundaryResource,
) -> Vec2 {
    let collision_force = get_separation_force(
        entity,
        *position,
        rotation,
        *velocity,
        spatial_index,
        query,
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
        query,
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
        query,
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
        query,
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

    acceleration = limit_vec2(acceleration, settings.max_force);

    // println!("velocity={:?} acceleration={:?}", velocity, acceleration);

    limit_vec2(*velocity + acceleration, settings.max_speed)
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
                    &query,
                    &settings,
                    &boundaries,
                ),
            )
        })
        .collect::<Vec<(Entity, Vec2)>>();

    for (entity, velocity) in velocities.into_iter() {
        if let Ok((_, _, mut icon_velocity)) = query.get_mut(entity) {
            icon_velocity.0 = velocity;
        }
    }

    debug!("update_icon_velocity in {:?}", start.elapsed());
}
