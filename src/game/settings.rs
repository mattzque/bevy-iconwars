use bevy::{ecs::system::Resource, reflect::Reflect};

#[derive(Resource, Debug, Clone, Reflect)]
pub struct SettingsResource {
    pub max_speed: f32,
    pub max_force: f32,

    pub max_icons: u32,

    pub avoidance_distance_dropzone: f32,
    pub avoidance_force_dropzone: f32,
    pub avoidance_force_bounds: f32,

    pub max_force_distance: f32,
    pub velocity_time_scale: f32,
    pub collision_distance: f32,
    pub separation_distance: f32,
    pub alignment_distance: f32,
    pub cohesion_distance: f32,
    pub separation_weight: f32,
    pub collision_weight: f32,
    pub alignment_weight: f32,
    pub cohesion_weight: f32,

    pub player_avoidance_distance: f32,
    pub player_avoidance_weight: f32,
    pub player_avoidance_max_speed: f32,
    pub player_avoidance_max_force: f32,

    pub seek_weight: f32,
    pub seek_max_speed: f32,
    pub seek_max_force: f32,

    pub controller_turn_speed: f32,
    pub controller_acceleration: f32,
    pub controller_dampening: f32,
    pub controller_max_speed: f32,

    pub max_hover_distance: f32,
    pub capture_time: f32,

    pub projectile_speed: f32,
    pub projectile_despawn_distance: f32,
    pub projectile_cooldown: f32,

    pub player_damage_amount: i32,
    pub player_damage_cooldown: f32,
    pub player_max_health: i32,

    // player gains additional points for bringing more followers to the dropzone at once
    pub player_score_follower_multiplier: f32,
    // player takes more damage the more followers they have
    pub player_damage_follower_multiplier: f32,
}

impl Default for SettingsResource {
    fn default() -> Self {
        Self {
            max_speed: 0.290,
            max_force: 0.05,

            max_icons: 0,

            avoidance_distance_dropzone: 87.0,
            avoidance_force_dropzone: 85.0,
            avoidance_force_bounds: 0.1,

            velocity_time_scale: 300.0,

            max_force_distance: 128.0,
            collision_distance: 77.0,
            separation_distance: 143.0,
            alignment_distance: 94.0,
            cohesion_distance: 102.0,

            separation_weight: 4.2,
            collision_weight: 3.1,
            alignment_weight: 2.95,
            cohesion_weight: 0.3,

            player_avoidance_distance: 128.0,
            player_avoidance_weight: 5.1,
            player_avoidance_max_speed: 1.2,
            player_avoidance_max_force: 0.1,

            seek_weight: 7.0,
            seek_max_speed: 0.78,
            seek_max_force: 0.08,

            controller_turn_speed: 3.85,
            controller_acceleration: 35.0,
            controller_dampening: 210.0,
            controller_max_speed: 100.0,

            max_hover_distance: 880.0,

            capture_time: 0.1,

            projectile_speed: 800.0,
            projectile_despawn_distance: 1024.0,
            projectile_cooldown: 0.3,

            player_damage_amount: 10,
            player_damage_cooldown: 0.5,

            player_max_health: 100,
            player_score_follower_multiplier: 0.1,
            player_damage_follower_multiplier: 0.5,
        }
    }
}
