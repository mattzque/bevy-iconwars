use std::time::Instant;

use bevy::gizmos;
use bevy::render::mesh::shape;
use bevy::render::view::NoFrustumCulling;
use bevy::utils::{HashMap, HashSet};
use rand::prelude::*;

use bevy::asset::RecursiveDependencyLoadState;
use bevy::prelude::*;

use super::assets::icons::IconSheetAsset;
use super::states::GameState;

mod renderer;

pub const ICON_SIZE: f32 = 32.0;
pub const ICON_MIN_DISTANCE: f32 = 45.25;

#[derive(Resource, Debug)]
pub struct IconSheetResource(pub Handle<IconSheetAsset>);

pub struct IconPlugin;

impl Plugin for IconPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::GameLoading), init_icons_system);
        app.add_systems(
            Update,
            debug_icons_system.run_if(in_state(GameState::GameLoading)),
        );
    }
}

/// Thats a LOT of entities!
#[derive(Component, Debug)]
pub struct IconEntity;

/// Render all entities using this single entity, thats not cheating, right? =)
#[derive(Component, Debug)]
pub struct IconRenderEntity;

#[derive(Component, Debug)]
pub struct IconInstanceData {
    /// Transforms of each icon, x, y and rotation.
    pub transforms: Vec<Vec3>,
}

impl IconInstanceData {
    pub fn new(transforms: Vec<Vec3>) -> Self {
        Self { transforms }
    }

    pub fn data(&self) -> Vec<u8> {
        let mut data = Vec::new();
        for Vec3 { x, y, z } in &self.transforms {
            data.extend_from_slice(&x.to_ne_bytes());
            data.extend_from_slice(&y.to_ne_bytes());
            data.extend_from_slice(&z.to_ne_bytes());
        }
        data
    }
}

#[derive(Component, Debug)]
pub struct IconSheetRef {
    pub sheet_index: usize,
    pub icon_index: usize,
    pub icon_name: String,
}

#[derive(Component, Debug)]
pub struct IconTransform {
    pub position: Vec2,
    pub rotation: f32,
}

// #[derive(Resource, Debug)]
// pub struct IconResource {
//     pub mesh: Handle<Mesh>,
// }

fn init_icons_system(
    mut commands: Commands,
    resource: Res<IconSheetResource>,
    assets: Res<Assets<IconSheetAsset>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let IconSheetAsset(sheets) = assets.get(&resource.0).unwrap();
    let mut rng = rand::thread_rng();
    let mut positions = Vec::new();
    let mut count = 0;
    let start = Instant::now();
    sheets.iter().enumerate().for_each(|(sheet_index, sheet)| {
        sheet
            .tiles
            .iter()
            .enumerate()
            .for_each(|(icon_index, icon)| {
                loop {
                    // candidate:
                    let position = Vec2::new(
                        rng.gen_range(-2000.0..2000.0),
                        rng.gen_range(-2000.0..2000.0),
                    );

                    // search for collisions:
                    let mut collision = false;
                    for (other_position, _, _) in &positions {
                        let d: Vec2 = position - *other_position;
                        if d.length() < ICON_MIN_DISTANCE {
                            collision = true;
                            break;
                        }
                    }
                    // try again if colliding with some other icon
                    if collision {
                        continue;
                    }

                    let rotation = (rng.gen_range(0.0..360.0) as f32).to_radians();
                    let components = (
                        IconEntity,
                        IconSheetRef {
                            sheet_index,
                            icon_index,
                            icon_name: icon.name.clone(),
                        },
                        IconTransform { position, rotation },
                    );
                    positions.push((position, rotation, components));
                    count += 1;
                    break;
                }
            });
    });
    info!("Fitted {} icons in {:?}", count, start.elapsed());

    let (transforms, entities): (Vec<Vec3>, Vec<Entity>) = positions
        .into_iter()
        .map(|(position, rotation, components)| {
            (
                Vec3::new(position.x, position.y, rotation),
                commands.spawn(components).id(),
            )
        })
        .unzip();

    let mesh = Mesh::from(shape::Plane {
        size: 1.0,
        subdivisions: 1,
    });
    let mesh_handle = meshes.add(mesh);
    commands.spawn((
        IconRenderEntity,
        mesh_handle,
        SpatialBundle {
            transform: Transform::from_translation(Vec3::ZERO),
            ..Default::default()
        },
        IconInstanceData::new(transforms),
        NoFrustumCulling,
    ));

    info!("Spawned {} icons", count);
}

fn debug_icons_system(query: Query<(&IconTransform)>, mut gizmos: Gizmos) {
    for IconTransform { position, rotation } in query.iter() {
        gizmos.rect_2d(*position, *rotation, Vec2::splat(ICON_SIZE), Color::RED);
        gizmos.circle_2d(*position, ICON_SIZE / 2.0, Color::BLUE);
    }
}
