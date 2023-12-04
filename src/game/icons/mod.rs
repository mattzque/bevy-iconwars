use std::time::Instant;

use bevy::prelude::*;
use bevy::render::mesh::shape;
use bevy::render::view::NoFrustumCulling;
use bevy::sprite::Mesh2dHandle;
use rand::prelude::*;

use super::assets::icons::IconSheetAsset;
use super::states::GameState;

mod renderer;

pub const ICON_SIZE: f32 = 32.0;
pub const ICON_MIN_DISTANCE: f32 = 45.25;

#[derive(Resource, Debug)]
pub struct IconSheetResource {
    pub handle: Handle<IconSheetAsset>,
    pub texture_array: Option<Handle<Image>>,
}

pub struct IconPlugin;

impl Plugin for IconPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(renderer::IconRendererPlugin);
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
    pub texture: Handle<Image>,
    /// Number of instances
    pub n_instances: u32,
    /// Transforms of each icon, x, y and rotation.
    pub transforms: Vec<Vec3>,
    /// References which sheet and the UV coordinate in the sheet
    pub indices: Vec<SheetIndex>,
}

#[derive(Debug)]
pub struct SheetIndex {
    pub sheet_index: u32,
    pub tile_uv: Vec2,
}

impl IconInstanceData {
    // vec3 (transform x, y, angle) + vec2 (uv) + uint (sheet index)
    pub const INSTANCE_LEN: u64 = ((std::mem::size_of::<f32>() * 3)
        + std::mem::size_of::<u32>()
        + (std::mem::size_of::<f32>() * 2)) as u64;

    pub fn new(texture: Handle<Image>, transforms: Vec<Vec3>, indices: Vec<SheetIndex>) -> Self {
        Self {
            texture,
            n_instances: transforms.len() as u32,
            transforms,
            indices,
        }
    }

    pub fn instances_data(&self) -> Vec<u8> {
        let mut data = Vec::new();
        for index in 0..self.n_instances {
            let Vec3 { x, y, z } = &self.transforms[index as usize];
            let SheetIndex {
                sheet_index,
                tile_uv,
            } = &self.indices[index as usize];

            let mut record = Vec::new();
            record.extend_from_slice(&x.to_le_bytes());
            record.extend_from_slice(&y.to_le_bytes());
            record.extend_from_slice(&z.to_le_bytes());
            record.extend_from_slice(&sheet_index.to_le_bytes());
            record.extend_from_slice(&tile_uv.x.to_le_bytes());
            record.extend_from_slice(&tile_uv.y.to_le_bytes());
            // println!("uv.z: {:?} {:?}", tile_uv.x, tile_uv.x.to_le_bytes());
            // println!("uv.x: {:?} {:?}", tile_uv.y, tile_uv.y.to_le_bytes());
            // println!("record: {:?}", record);
            // println!("record: {:?}", record.len());
            // assert_eq!(record.len(), Self::INSTANCE_LEN as usize);
            data.extend_from_slice(&record);
        }
        // println!("indices: {:?}", self.indices);
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

fn init_icons_system(
    mut commands: Commands,
    resource: Res<IconSheetResource>,
    assets: Res<Assets<IconSheetAsset>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let IconSheetAsset(sheets) = assets.get(&resource.handle).unwrap();
    let mut rng = rand::thread_rng();
    let mut positions = Vec::new();
    let mut textures = Vec::new();
    let mut indices = Vec::new();
    let mut count = 0;
    let start = Instant::now();
    sheets.iter().enumerate().for_each(|(sheet_index, sheet)| {
        textures.push(sheet.handle.clone());
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
                    indices.push(SheetIndex {
                        sheet_index: sheet_index as u32,
                        tile_uv: Vec2::new(
                            icon.x as f32, // / sheet.width as f32,
                            icon.y as f32, // / sheet.height as f32,
                        ),
                    });
                    count += 1;
                    break;
                }
            });
    });
    info!("Fitted {} icons in {:?}", count, start.elapsed());

    let (transforms, _entities): (Vec<Vec3>, Vec<Entity>) = positions
        .into_iter()
        .map(|(position, rotation, components)| {
            (
                Vec3::new(position.x, position.y, rotation),
                commands.spawn(components).id(),
            )
        })
        .unzip();

    info!("Spawned {} icons", count);

    let mesh = Mesh::from(shape::Quad {
        size: Vec2::splat(ICON_SIZE),
        flip: false,
    });
    let mesh_handle = meshes.add(mesh);
    commands.spawn((
        IconRenderEntity,
        Mesh2dHandle(mesh_handle),
        SpatialBundle {
            transform: Transform::from_translation(Vec3::ZERO),
            ..Default::default()
        },
        IconInstanceData::new(resource.texture_array.clone().unwrap(), transforms, indices),
        NoFrustumCulling,
    ));
}

fn debug_icons_system(query: Query<&IconTransform>, mut gizmos: Gizmos) {
    for IconTransform { position, rotation } in query.iter() {
        gizmos.rect_2d(*position, *rotation, Vec2::splat(ICON_SIZE), Color::RED);
        gizmos.circle_2d(*position, ICON_SIZE / 2.0, Color::BLUE);
    }
}
