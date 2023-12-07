use bevy::{ecs::system::Resource, reflect::Reflect};

#[derive(Resource, Debug, Clone, Reflect)]
pub struct SettingsResource {
    pub max_speed: f32,
    pub max_force: f32,

    pub avoidance_distance_dropzone: f32,
    pub avoidance_force_dropzone: f32,
    pub avoidance_force_bounds: f32,

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
    pub seek_max_speed: f32,
    pub seek_max_force: f32,

    pub controller_turn_speed: f32,
    pub controller_acceleration: f32,
    pub controller_dampening: f32,
    pub controller_max_speed: f32,

    pub max_hover_distance: f32,
    pub capture_time: f32,
}

impl Default for SettingsResource {
    fn default() -> Self {
        Self {
            max_speed: 0.290,
            max_force: 0.05,

            avoidance_distance_dropzone: 87.0,
            avoidance_force_dropzone: 85.0,
            avoidance_force_bounds: 0.1,

            velocity_time_scale: 300.0,

            collision_distance: 77.0,
            separation_distance: 143.0,
            alignment_distance: 94.0,
            cohesion_distance: 102.0,

            separation_weight: 4.2,
            collision_weight: 3.1,
            alignment_weight: 2.95,
            cohesion_weight: 0.3,

            seek_weight: 7.0,
            seek_max_speed: 0.78,
            seek_max_force: 0.08,

            controller_turn_speed: 3.85,
            controller_acceleration: 35.0,
            controller_dampening: 210.0,
            controller_max_speed: 100.0,

            max_hover_distance: 880.0,

            capture_time: 0.1,
        }
    }
}
