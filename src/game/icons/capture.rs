use bevy::prelude::*;
use bevy::utils::HashSet;
use bevy::{utils::Instant, window::PrimaryWindow};
use bevy_prototype_lyon::prelude::*;

use crate::game::camera::CameraTag;
use crate::game::world::WorldBoundaryResource;
use crate::game::{settings::SettingsResource, states::GameState};

use super::commands::{CircleShapeCommand, LineShapeCommand};
use super::components::{
    IconCaptureProgressLine, IconFollowerCircle, IconFollowerLine, IconType, Type,
};
use super::resources::HoveredIcon;
use super::ICON_CIRCLE_RADIUS;
use super::{
    components::{IconHoveredCircle, IconTransform},
    resources::SpatialIndexResource,
    IconPlayerController, ICON_SIZE,
};

#[derive(Resource, Debug, Default)]
struct IconCaptureProgress {
    pub entity: Option<Entity>,
    pub progress: f32,
    pub start: Option<Instant>,
}

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

        let perfect_grid_offset = (cols * cols) - n_icons as i32;
        if perfect_grid_offset > 0 {
            println!(
                "perfect grid would need {} more icons!",
                perfect_grid_offset
            );
        }

        // (Vec2::new(x, y), icon_size / ICON_SIZE)
        Vec2::new(x, y)
    }
}

#[derive(Resource, Debug, Default)]
struct IconFollowers {
    pub followers: HashSet<Entity>,
}

pub struct IconCapturePlugin;

impl Plugin for IconCapturePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(IconCaptureProgress::default());
        app.insert_resource(IconCapturedGrid::default());
        app.insert_resource(IconFollowers::default());
        app.add_systems(OnEnter(GameState::GameLoading), init_capture);
        app.add_systems(
            Update,
            (
                update_hovered_icon_system,
                start_capture_on_mouse_down,
                stop_capture_on_mouse_leave,
                progress_capture_timer,
                update_capture_progress_line,
                update_follower_paths,
                player_follower_dropzone,
            )
                .run_if(in_state(GameState::GameRunning)),
        );
    }
}

fn init_capture(mut commands: Commands) {
    commands.add(CircleShapeCommand::<IconHoveredCircle> {
        radius: ICON_CIRCLE_RADIUS,
        z: 1.0,
        color: "#884c56",
        stroke_width: 4.0,
        visibility: Visibility::Hidden,
        ..Default::default()
    });
    commands.add(LineShapeCommand::<IconCaptureProgressLine> {
        z: 1.0,
        color: "#884c56",
        stroke_width: 4.0,
        visibility: Visibility::Hidden,
        ..Default::default()
    });
}

#[allow(clippy::too_many_arguments)]
fn update_hovered_icon_system(
    mut commands: Commands,
    boundaries: Res<WorldBoundaryResource>,
    window: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform), With<CameraTag>>,
    index: Res<SpatialIndexResource>,
    player: Query<(Entity, &IconTransform), With<IconPlayerController>>,
    icon_types: Query<&IconType, Without<IconPlayerController>>,
    mut hovered_circle: Query<(&mut Visibility, &mut Transform), With<IconHoveredCircle>>,
    settings: Res<SettingsResource>,
) {
    let (camera, camera_transform) = camera.single();
    let window = window.single();
    let (_player, player_transform) = player.single();
    // player cant hover items while she is in the drop zone!
    if boundaries.in_dropzone(player_transform.position) {
        return;
    }

    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
    {
        commands.insert_resource(HoveredIcon(None));
        let (mut visibility, _) = hovered_circle.single_mut();
        *visibility = Visibility::Hidden;

        let threshold = 16.0;
        if let Some(result) = index
            .0
            .query(world_position, ICON_SIZE / 2.0 + threshold)
            .next()
        {
            let distance_to_player = player_transform.position.distance(*result.position);
            let icon_type = icon_types
                .get(result.key)
                .map(|result| result.0)
                .unwrap_or(Type::Player);

            // do not set hover on icons that are one of:
            //  - is not a free moving icon
            //  - too far away from the player
            if icon_type != Type::Free || distance_to_player > settings.max_hover_distance {
                return;
            }

            commands.insert_resource(HoveredIcon(Some(result.key)));
            let (mut visibility, mut transform) = hovered_circle.single_mut();
            transform.translation.x = result.position.x;
            transform.translation.y = result.position.y;
            *visibility = Visibility::Visible;
        }
    }
}

fn start_capture_on_mouse_down(
    mouse_button_input: Res<Input<MouseButton>>,
    hovered_icon: Res<HoveredIcon>,
    mut capture_progress: ResMut<IconCaptureProgress>,
) {
    if mouse_button_input.just_pressed(MouseButton::Left) {
        if let Some(hovered_icon) = hovered_icon.0 {
            // if the icon is already capturing do nothing (perhaps increase progress? or show some visual pulse animation or something?)
            if capture_progress.entity == Some(hovered_icon) {
                return;
            }

            capture_progress.entity = Some(hovered_icon);
            capture_progress.progress = 0.0;
            capture_progress.start = Some(Instant::now());
        }
    }
}

fn stop_capture_on_mouse_leave(
    hovered_icon: Res<HoveredIcon>,
    mut capture_progress: ResMut<IconCaptureProgress>,
) {
    if let Some(entity) = capture_progress.entity {
        if let Some(hovered) = hovered_icon.0 {
            if hovered != entity {
                capture_progress.entity = None;
            }
        } else {
            capture_progress.entity = None;
        }
    }
}

fn progress_capture_timer(
    mut capture_progress: ResMut<IconCaptureProgress>,
    settings: Res<SettingsResource>,
    mut followers: ResMut<IconFollowers>,
    mut icons: Query<&mut IconType>,
    mut hovered_icon: ResMut<HoveredIcon>,
) {
    if let Some(entity) = capture_progress.entity {
        let now = Instant::now();
        let duration = now.duration_since(capture_progress.start.unwrap());
        let progress = duration.as_secs_f32() / settings.capture_time;
        capture_progress.progress = progress;

        if progress > 1.0 {
            followers.followers.insert(entity);
            capture_progress.entity = None;
            icons.get_mut(entity).unwrap().0 = Type::Follower;
            hovered_icon.0 = None;
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

// draw line from player to icon that is being captured
fn update_capture_progress_line(
    mut line: Query<(&mut Path, &mut Visibility), With<IconCaptureProgressLine>>,
    capture_progress: Res<IconCaptureProgress>,
    player: Query<&IconTransform, With<IconPlayerController>>,
    query: Query<&IconTransform>,
) {
    let mut line = line.single_mut();
    if let Some(entity) = capture_progress.entity {
        let player_transform = player.single();
        let transform = query.get(entity).unwrap();

        let (start, end) = get_line_points(player_transform.position, transform.position);

        let mut builder = GeometryBuilder::new();
        builder = builder.add(&shapes::Line(start, end));
        *line.0 = builder.build();
        *line.1 = Visibility::Visible;
    } else {
        *line.1 = Visibility::Hidden;
    }
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
                z: 0.9,
                color: "#884c56",
                stroke_width: 4.0,
                visibility: Visibility::Visible,
                tag: IconFollowerLine(*follower),
            });

            commands.add(CircleShapeCommand::<IconFollowerCircle> {
                radius: ICON_CIRCLE_RADIUS,
                position: transform.position,
                z: 0.9,
                color: "#884c56",
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
fn player_follower_dropzone(
    boundaries: Res<WorldBoundaryResource>,
    mut followers: ResMut<IconFollowers>,
    mut icons: Query<(Entity, &mut IconTransform, &mut IconType), Without<IconPlayerController>>,
    player: Query<&IconTransform, With<IconPlayerController>>,
    mut captured: ResMut<IconCapturedGrid>,
    mut spatial_index: ResMut<SpatialIndexResource>,
) {
    let position = player.single().position;

    // icons.iter_mut().for_each(|(entity, _, mut icon_type)| {
    //     icon_type.0 = Type::Follower;
    //     followers.followers.insert(entity);
    // });

    if boundaries.in_dropzone(position) && !followers.followers.is_empty() {
        let icon_count = icons.iter().count();
        for follower in followers.followers.iter() {
            let mut icon = icons.get_mut(*follower).unwrap();

            let new_position = captured.add_captured(*follower, &boundaries, icon_count);

            // TODO animate the position to target?
            icon.1.position = new_position; //  Vec2::ZERO; // TODO!
            icon.1.rotation = 0.0;
            icon.2 .0 = Type::Captured;
            println!("captured icon! {:?}", follower);

            // update position in spatial index, or just remove it?
            spatial_index.0.insert(*follower, new_position, Vec2::ZERO);
        }
        followers.followers.clear();
    }
}
