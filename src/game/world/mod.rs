use super::camera::{CAMERA_LAYER, CAMERA_Z_BACKGROUND};
use super::states::GameState;
use bevy::prelude::*;
use bevy::render::batching::NoAutomaticBatching;
use bevy::render::view::RenderLayers;
use bevy_prototype_lyon::prelude::*;

#[derive(Resource)]
pub struct WorldBoundaryResource {
    pub grid_spacing: f32,
    /// Upper Left corner of the game world
    pub bounds_min: Vec2,
    /// Lower Right corner of the game world
    pub bounds_max: Vec2,
    /// Upper Left corner of the drop zone
    pub dropzone_min: Vec2,
    /// Lower Right corner of the drop zone
    pub dropzone_max: Vec2,
}

impl Default for WorldBoundaryResource {
    fn default() -> Self {
        let world = 1024.0 * 5.0;
        let dropzone = 1024.0;

        Self {
            grid_spacing: 512.0,
            bounds_min: Vec2::splat(-world),
            bounds_max: Vec2::splat(world),
            dropzone_min: Vec2::splat(-dropzone),
            dropzone_max: Vec2::splat(dropzone),
        }
    }
}

impl WorldBoundaryResource {
    /// Returns true if the point lies within the dropzone min/max bounds
    pub fn in_dropzone(&self, point: Vec2) -> bool {
        point.x >= self.dropzone_min.x
            && point.x <= self.dropzone_max.x
            && point.y >= self.dropzone_min.y
            && point.y <= self.dropzone_max.y
    }
}

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WorldBoundaryResource::default());
        app.add_plugins(ShapePlugin);
        // app.add_systems(OnEnter(GameState::GameLoading), setup_world_grid);
        app.add_systems(OnEnter(GameState::GameLoading), setup_world_grid);
    }
}

fn setup_world_grid(mut commands: Commands, boundary: Res<WorldBoundaryResource>) {
    let width = boundary.bounds_max.x - boundary.bounds_min.x;
    let height = boundary.bounds_max.y - boundary.bounds_min.y;

    let mut builder = GeometryBuilder::new();
    builder = builder.add(&shapes::Rectangle {
        extents: Vec2::new(width, height),
        origin: RectangleOrigin::Center,
    });

    let mut x = boundary.bounds_min.x;
    while x < boundary.bounds_max.x {
        builder = builder.add(&shapes::Line(
            Vec2::new(x, boundary.bounds_min.y),
            Vec2::new(x, boundary.bounds_max.y),
        ));
        x += boundary.grid_spacing;
    }

    let mut y = boundary.bounds_min.y;
    while y < boundary.bounds_max.y {
        builder = builder.add(&shapes::Line(
            Vec2::new(boundary.bounds_min.x, y),
            Vec2::new(boundary.bounds_max.x, y),
        ));
        y += boundary.grid_spacing;
    }

    commands.spawn((
        ShapeBundle {
            path: builder.build(),
            spatial: SpatialBundle {
                transform: Transform::from_translation(Vec3::new(
                    0.0,
                    0.0,
                    CAMERA_Z_BACKGROUND - 1.0,
                )),
                ..Default::default()
            },
            ..Default::default()
        },
        Fill::color(Color::hex("#22272e").unwrap()),
        Stroke::new(Color::hex("#444c56").unwrap(), 1.0),
        RenderLayers::layer(CAMERA_LAYER),
        NoAutomaticBatching,
    ));

    let mut builder = GeometryBuilder::new();
    let width = boundary.dropzone_max.x - boundary.dropzone_min.x;
    let height = boundary.dropzone_max.y - boundary.dropzone_min.y;
    builder = builder.add(&shapes::Rectangle {
        extents: Vec2::new(width, height),
        origin: RectangleOrigin::Center,
    });

    commands.spawn((
        ShapeBundle {
            path: builder.build(),
            spatial: SpatialBundle {
                transform: Transform::from_translation(Vec3::new(0.0, 0.0, CAMERA_Z_BACKGROUND)),
                ..Default::default()
            },
            ..Default::default()
        },
        Fill::color(Color::hex("#2d333b").unwrap()),
        Stroke::new(Color::hex("#444c56").unwrap(), 1.0),
        RenderLayers::layer(CAMERA_LAYER),
        NoAutomaticBatching,
    ));
}
