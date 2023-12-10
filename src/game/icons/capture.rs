use bevy::prelude::*;
use bevy::utils::HashSet;
use bevy_prototype_lyon::prelude::*;

use crate::game::world::WorldBoundaryResource;
use crate::game::{settings::SettingsResource, states::GameState};

use super::commands::{CircleShapeCommand, LineShapeCommand};
use super::components::{IconFollowerCircle, IconFollowerLine, IconType, Type};
use super::events::{IconCaptureEvent, PlayerFollowEvent, ProjectileSpawnEvent};
use super::health::PlayerScore;
use super::ICON_CIRCLE_RADIUS;
use super::{
    components::IconTransform, resources::SpatialIndexResource, IconPlayerController, ICON_SIZE,
};

#[derive(Resource, Debug, Default)]
struct IconCapturedGrid {
    pub captured: Vec<Entity>,
}

impl IconCapturedGrid {
    // add captured icon to the dropzone grid and return its location and new scale
    pub fn add_captured(
        &mut self,
        entity: Entity,
        boundaries: &WorldBoundaryResource,
        n_icons: usize,
    ) -> Vec2 {
        let index = self.captured.len() as i32;
        self.captured.push(entity);

        let min = boundaries.dropzone_min;
        let max = boundaries.dropzone_max;
        let size = max - min;
        assert_eq!(size.x, size.y);
        let size = size.x; // something like 2048
        let cols = f32::sqrt(n_icons as f32).ceil() as i32;
        let icon_size = size / cols as f32;
        let x = (min.x + (index % cols) as f32 * icon_size) + icon_size / 2.0;
        let y = (min.y + (cols - (index / cols) - 1) as f32 * icon_size) + icon_size / 2.0;

        // let perfect_grid_offset = (cols * cols) - n_icons as i32;
        // if perfect_grid_offset > 0 {
        //     println!(
        //         "perfect grid would need {} more icons!",
        //         perfect_grid_offset
        //     );
        // }

        // (Vec2::new(x, y), icon_size / ICON_SIZE)
        Vec2::new(x, y)
    }
}

#[derive(Resource, Debug, Default)]
pub struct IconFollowers {
    pub followers: HashSet<Entity>,
}

pub struct IconCapturePlugin;

impl Plugin for IconCapturePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerFollowEvent>();
        app.add_event::<ProjectileSpawnEvent>();
        app.add_event::<IconCaptureEvent>();
        app.insert_resource(IconCapturedGrid::default());
        app.insert_resource(ProjectileCooldown::default());
        app.insert_resource(IconFollowers::default());

        app.add_systems(OnEnter(GameState::GameOver), despawn_game_over);
        app.add_systems(OnEnter(GameState::MainMenu), despawn_game_over);
        app.add_systems(
            Update,
            (
                spawn_projectile_system,
                update_projectiles_system,
                update_follower_paths,
                player_follower_dropzone,
            )
                .run_if(in_state(GameState::GameRunning)),
        );
    }
}

#[derive(Component, Debug, Default)]
pub struct Projectile {
    pub start: Vec2,    // despawn after a certain distance
    pub velocity: Vec2, // indicates direction and speed
}

#[derive(Resource, Default)]
pub struct ProjectileCooldown {
    pub timer: Option<Timer>,
}

#[allow(clippy::too_many_arguments)]
fn spawn_projectile_system(
    mut commands: Commands,
    player: Query<&IconTransform, With<IconPlayerController>>,
    followers: Res<IconFollowers>,
    keys: Res<Input<KeyCode>>,
    mouse_button_input: Res<Input<MouseButton>>,
    settings: Res<SettingsResource>,
    boundaries: Res<WorldBoundaryResource>,
    time: Res<Time>,
    mut cooldown: ResMut<ProjectileCooldown>,
    mut events: EventWriter<ProjectileSpawnEvent>,
) {
    if let Some(timer) = cooldown.timer.as_mut() {
        timer.tick(time.delta());
        if !timer.finished() {
            return;
        }
    }

    if keys.just_pressed(KeyCode::Space) || mouse_button_input.just_pressed(MouseButton::Left) {
        let n_projectiles = (1 + followers.followers.len()).min(20);

        let player = player.single();

        // no shooting in the dropzone!
        if boundaries.in_dropzone(player.position) {
            // return;
        }

        cooldown.timer = Some(Timer::from_seconds(
            settings.projectile_cooldown,
            TimerMode::Once,
        ));

        let spread = 45.0_f32.to_radians();
        let half_spread = spread / 2.0;
        let step = spread / n_projectiles as f32;

        for i in 0..n_projectiles {
            let rotation = (i as f32) * step + (step / 2.0);
            let rotation = (player.rotation + std::f32::consts::PI / 2.0) + rotation - half_spread;

            let direction = Vec2::new(rotation.cos(), rotation.sin());
            let start = player.position + direction * ICON_CIRCLE_RADIUS;

            events.send(ProjectileSpawnEvent);

            commands.add(CircleShapeCommand {
                radius: 8.0,
                position: start,
                stroke_width: 1.0,
                color: "#7fc1bb",
                fill_color: Some("#7fc1bb"),
                tag: Projectile {
                    start,
                    velocity: direction * settings.projectile_speed,
                },
                ..Default::default()
            });
        }
    }
}
// float angleStep = SpreadAngle / NumberOfProjectiles;
//         float aimingAngle = AimOrigin.rotation.eulerAngles.z;
//         float centeringOffset = (SpreadAngle / 2) - (angleStep / 2); //offsets every projectile so the spread is                                                                                                                         //centered on the mouse cursor

//         for (int i = 0; i < NumberOfProjectiles; i++)
//         {
//             float currentBulletAngle = angleStep * i;

//             Quaternion rotation = Quaternion.Euler(new Vector3(0, 0, aimingAngle + currentBulletAngle - centeringOffset));
//             GameObject bullet = Instantiate(BulletPrefab, ProjectileSpawnPosition.position, rotation);

#[allow(clippy::too_many_arguments)]
fn update_projectiles_system(
    mut commands: Commands,
    mut projectiles: Query<(Entity, &mut Transform, &Projectile)>,
    mut icons: Query<&mut IconType>,
    index: Res<SpatialIndexResource>,
    time: Res<Time>,
    settings: Res<SettingsResource>,
    boundaries: Res<WorldBoundaryResource>,
    mut followers: ResMut<IconFollowers>,
    mut events: EventWriter<PlayerFollowEvent>,
) {
    let dt = time.delta_seconds();
    for (entity, mut transform, Projectile { start, velocity }) in projectiles.iter_mut() {
        transform.translation += Vec3::new(velocity.x, velocity.y, 0.0) * dt;
        let position = Vec2::new(transform.translation.x, transform.translation.y);

        // despawn in dropzone
        if boundaries.in_dropzone(position) {
            commands.entity(entity).despawn();
            continue;
        }

        let distance = transform
            .translation
            .distance(Vec3::new(start.x, start.y, 0.0));
        if distance >= settings.projectile_despawn_distance {
            commands.entity(entity).despawn();
            continue;
        }

        // find something that intersects with the projectile:
        for result in index.0.query(position, ICON_SIZE / 2.0) {
            let mut icon_type = icons.get_mut(result.key).unwrap();
            if icon_type.0 == Type::Free {
                // icon becomes a follower!
                followers.followers.insert(result.key);
                events.send(PlayerFollowEvent { entity: result.key });

                // despawn projectile:
                commands.entity(entity).despawn();

                icon_type.0 = Type::Follower;
                break;
            }
        }
    }
}

fn get_line_points(start: Vec2, end: Vec2) -> (Vec2, Vec2) {
    let mut start = start;
    let mut end = end;

    let direction = (end - start).normalize();
    start += direction * ICON_CIRCLE_RADIUS;
    end -= direction * ICON_CIRCLE_RADIUS;

    (start, end)
}

fn update_follower_paths(
    mut commands: Commands,
    mut lines: Query<(Entity, &mut Path, &IconFollowerLine)>,
    mut circles: Query<(Entity, &mut Transform, &IconFollowerCircle)>,
    player: Query<&IconTransform, With<IconPlayerController>>,
    icons: Query<&IconTransform>,
    followers: Res<IconFollowers>,
) {
    let player = player.single().position;

    let mut existing = HashSet::new();

    // update existing lines & circles
    for (shape_entity, mut path, line) in lines.iter_mut() {
        let IconFollowerLine(entity) = line;
        if followers.followers.contains(entity) {
            let transform = icons.get(*entity).unwrap();
            let (start, end) = get_line_points(player, transform.position);
            let mut builder = GeometryBuilder::new();
            builder = builder.add(&shapes::Line(start, end));
            *path = builder.build();
            existing.insert(entity);
        } else {
            commands.entity(shape_entity).despawn();
        }
    }
    for (shape_entity, mut circle_transform, circle) in circles.iter_mut() {
        let IconFollowerCircle(entity) = circle;
        if followers.followers.contains(entity) {
            let transform = icons.get(*entity).unwrap();
            circle_transform.translation.x = transform.position.x;
            circle_transform.translation.y = transform.position.y;
        } else {
            commands.entity(shape_entity).despawn();
        }
    }

    // add lines & circles for new followers
    for follower in &followers.followers {
        if !existing.contains(&follower) {
            let transform = icons.get(*follower).unwrap();
            let (start, end) = get_line_points(player, transform.position);
            commands.add(LineShapeCommand::<IconFollowerLine> {
                start,
                end,
                color: "#884c56",
                stroke_width: 4.0,
                visibility: Visibility::Visible,
                tag: IconFollowerLine(*follower),
            });

            commands.add(CircleShapeCommand::<IconFollowerCircle> {
                radius: ICON_CIRCLE_RADIUS,
                position: transform.position,
                color: "#884c56",
                fill_color: None,
                stroke_width: 4.0,
                visibility: Visibility::Visible,
                tag: IconFollowerCircle(*follower),
            });
        }
    }
}

// when player moves into drop zone / or is in the drop zone
// all follower icons are put into the drop zone and no longer move at all,
// arranging the icons in a grid
#[allow(clippy::too_many_arguments)]
fn player_follower_dropzone(
    boundaries: Res<WorldBoundaryResource>,
    mut followers: ResMut<IconFollowers>,
    mut icons: Query<(Entity, &mut IconTransform, &mut IconType), Without<IconPlayerController>>,
    player: Query<&IconTransform, With<IconPlayerController>>,
    mut captured: ResMut<IconCapturedGrid>,
    mut spatial_index: ResMut<SpatialIndexResource>,
    mut events: EventWriter<IconCaptureEvent>,
    mut score: ResMut<PlayerScore>,
    settings: Res<SettingsResource>,
    mut state: ResMut<NextState<GameState>>,
) {
    let position = player.single().position;

    if boundaries.in_dropzone(position) && !followers.followers.is_empty() {
        let icon_count = icons.iter().count();
        let mut n_events_sent = 0;
        for follower in followers.followers.iter() {
            let mut icon = icons.get_mut(*follower).unwrap();

            let new_position = captured.add_captured(*follower, &boundaries, icon_count);

            // TODO animate the position to target?
            icon.1.position = new_position; //  Vec2::ZERO; // TODO!
            icon.1.rotation = 0.0;
            icon.2 .0 = Type::Captured;

            score.score += 1
                + (followers.followers.len() as f32 * settings.player_score_follower_multiplier)
                    as u32;

            if n_events_sent < 10 {
                events.send(IconCaptureEvent { entity: *follower });
                n_events_sent += 1;
            }

            // update position in spatial index, or just remove it?
            spatial_index.0.insert(*follower, new_position, Vec2::ZERO);

            // check for win condition:
            let is_winner = icons.iter().all(|(_, _, icon_type)| {
                icon_type.0 == Type::Player || icon_type.0 == Type::Captured
            });
            if is_winner {
                state.set(GameState::GameOver);
            }
        }
        followers.followers.clear();
    }
}

fn despawn_game_over(
    mut commands: Commands,
    lines: Query<Entity, With<IconFollowerLine>>,
    circles: Query<Entity, With<IconFollowerCircle>>,
    projectiles: Query<Entity, With<Projectile>>,
    mut followers: ResMut<IconFollowers>,
) {
    followers.followers.clear();
    commands.insert_resource(IconCapturedGrid::default());
    commands.insert_resource(ProjectileCooldown::default());
    commands.insert_resource(IconFollowers::default());

    // despawn all of them:
    for entity in lines.iter().chain(circles.iter()).chain(projectiles.iter()) {
        commands.entity(entity).despawn_recursive();
    }
}
