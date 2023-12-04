use bevy::prelude::*;

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
            data.extend_from_slice(&record);
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
