use bevy::{ecs::system::Resource, reflect::Reflect};

#[derive(Resource, Debug, Clone, Reflect)]
pub struct SettingsResource {
    pub max_speed: f32,
    pub max_force: f32,
    pub velocity_time_scale: f32,
    pub collision_distance: f32,
    pub separation_distance: f32,
    pub alignment_distance: f32,
    pub cohesion_distance: f32,
    pub separation_weight: f32,
    pub collision_weight: f32,
    pub alignment_weight: f32,
    pub cohesion_weight: f32,
    pub seek_weight: f32,
}

impl Default for SettingsResource {
    fn default() -> Self {
        Self {
            max_speed: 0.290,
            max_force: 0.05,

            velocity_time_scale: 300.0,

            collision_distance: 77.0,
            separation_distance: 143.0,
            alignment_distance: 94.0,
            cohesion_distance: 102.0,

            separation_weight: 4.2,
            collision_weight: 3.1,
            alignment_weight: 2.95,
            cohesion_weight: 0.3,

            seek_weight: 0.1,
        }
    }
}
