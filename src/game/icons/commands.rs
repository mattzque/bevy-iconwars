use bevy::ecs::system::Command;
use bevy::prelude::*;
use bevy::render::view::NoFrustumCulling;
use bevy_prototype_lyon::prelude::*;

use super::ICON_CIRCLE_RADIUS;

pub struct CircleShapeCommand<T> {
    pub radius: f32,
    pub position: Vec2,
    pub stroke_width: f32,
    pub z: f32,
    pub visibility: Visibility,
    pub color: &'static str,
    pub tag: T,
}

impl<T> Default for CircleShapeCommand<T>
where
    T: Default,
{
    fn default() -> Self {
        Self {
            radius: ICON_CIRCLE_RADIUS,
            position: Vec2::ZERO,
            stroke_width: 1.0,
            z: 1.0,
            visibility: Visibility::Visible,
            color: "#ffffff",
            tag: T::default(),
        }
    }
}

impl<T> Command for CircleShapeCommand<T>
where
    T: Component,
{
    fn apply(self, world: &mut World) {
        let mut builder = GeometryBuilder::new();
        builder = builder.add(&shapes::Circle {
            radius: self.radius,
            center: Vec2::ZERO,
        });
        world.spawn((
            ShapeBundle {
                path: builder.build(),
                spatial: SpatialBundle {
                    transform: Transform::from_translation(Vec3::new(
                        self.position.x,
                        self.position.y,
                        self.z,
                    )),
                    visibility: self.visibility,
                    ..Default::default()
                },
                ..Default::default()
            },
            Stroke::new(Color::hex(self.color).unwrap(), self.stroke_width),
            self.tag,
            NoFrustumCulling,
        ));
    }
}

pub struct LineShapeCommand<T> {
    pub start: Vec2,
    pub end: Vec2,
    pub stroke_width: f32,
    pub z: f32,
    pub visibility: Visibility,
    pub color: &'static str,
    pub tag: T,
}

impl<T> Default for LineShapeCommand<T>
where
    T: Default,
{
    fn default() -> Self {
        Self {
            start: Vec2::ZERO,
            end: Vec2::ZERO,
            stroke_width: 1.0,
            z: 1.0,
            visibility: Visibility::Visible,
            color: "#ffffff",
            tag: T::default(),
        }
    }
}

impl<T> Command for LineShapeCommand<T>
where
    T: Component + Send + Sync + 'static,
{
    fn apply(self, world: &mut World) {
        let mut builder = GeometryBuilder::new();
        builder = builder.add(&shapes::Line(self.start, self.end));
        world.spawn((
            ShapeBundle {
                path: builder.build(),
                spatial: SpatialBundle {
                    transform: Transform::from_translation(Vec3::new(0.0, 0.0, self.z)),
                    visibility: self.visibility,
                    ..Default::default()
                },
                ..Default::default()
            },
            Stroke::new(Color::hex(self.color).unwrap(), self.stroke_width),
            NoFrustumCulling,
            self.tag,
        ));
    }
}
