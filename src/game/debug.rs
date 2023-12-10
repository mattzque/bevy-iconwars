use bevy::prelude::*;
use bevy_egui::{
    egui::{self, CursorIcon},
    EguiContexts, EguiPlugin,
};

use super::{settings::SettingsResource, states::GameState};

#[derive(Resource, Default)]
pub struct ShowDebug {
    show: bool,
}

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ShowDebug::default());
        app.add_plugins(EguiPlugin);
        app.add_systems(Update, render_settings_gui);
        app.add_systems(Update, toggle_debug);
    }
}

pub fn toggle_debug(keys: Res<Input<KeyCode>>, mut show_debug: ResMut<ShowDebug>) {
    if keys.just_pressed(KeyCode::O) {
        show_debug.show = !show_debug.show;
    }
}

pub fn render_settings_gui(
    mut settings: ResMut<SettingsResource>,
    mut contexts: EguiContexts,
    show_debug: Res<ShowDebug>,
    state: Res<State<GameState>>,
) {
    if *state == GameState::GameRunning {
        contexts.ctx_mut().output_mut(|o| {
            o.cursor_icon = CursorIcon::Crosshair;
        });
    } else {
        contexts.ctx_mut().output_mut(|o| {
            o.cursor_icon = CursorIcon::Default;
        });
    }

    if !show_debug.show {
        return;
    }
    egui::Window::new("SettingsResource").show(contexts.ctx_mut(), |ui| {
        ui.style_mut().spacing.slider_width = 300.0;

        ui.add(egui::Slider::new(&mut settings.max_icons, 0..=2000).text("Max Icons (0 = all)"));

        ui.add(egui::Slider::new(&mut settings.max_speed, 0.0..=2.0).text("Max Speed"));
        ui.add(egui::Slider::new(&mut settings.max_force, 0.0..=2.0).text("Max Force"));
        ui.add(
            egui::Slider::new(&mut settings.avoidance_force_bounds, 0.0..=100.0)
                .text("Avoidance Force: Bounds"),
        );
        ui.add(
            egui::Slider::new(&mut settings.avoidance_force_dropzone, 0.0..=100.0)
                .text("Avoidance Force: Dropzone"),
        );
        ui.add(
            egui::Slider::new(&mut settings.avoidance_distance_dropzone, 0.0..=500.0)
                .text("Avoidance Distance: Dropzone "),
        );
        ui.add(
            egui::Slider::new(&mut settings.velocity_time_scale, 0.0..=4000.0)
                .text("Velocity Time Scale"),
        );
        ui.add(
            egui::Slider::new(&mut settings.max_force_distance, 32.0..=256.0)
                .text("Max Distance (px)"),
        );
        ui.add(
            egui::Slider::new(&mut settings.collision_distance, 5.0..=150.0)
                .text("Collision Distance (px)"),
        );
        ui.add(
            egui::Slider::new(&mut settings.separation_distance, 5.0..=150.0)
                .text("Separation Distance (px)"),
        );
        ui.add(
            egui::Slider::new(&mut settings.alignment_distance, 5.0..=150.0)
                .text("Alignment Distance (px)"),
        );
        ui.add(
            egui::Slider::new(&mut settings.cohesion_distance, 5.0..=150.0)
                .text("Cohesion Distance (px)"),
        );

        ui.add(
            egui::Slider::new(&mut settings.player_avoidance_distance, 5.0..=150.0)
                .text("Player Avoidance Distance (px)"),
        );
        ui.add(
            egui::Slider::new(&mut settings.player_avoidance_weight, 0.0..=10.0)
                .text("Player Avoidance Weight"),
        );
        ui.add(
            egui::Slider::new(&mut settings.player_avoidance_max_speed, 0.0..=2.0)
                .text("Player Avoidance: Max Speed"),
        );
        ui.add(
            egui::Slider::new(&mut settings.player_avoidance_max_force, 0.0..=2.0)
                .text("Player Avoidance: Max Force"),
        );

        ui.add(
            egui::Slider::new(&mut settings.separation_weight, 0.0..=10.0)
                .text("Separation Weight"),
        );
        ui.add(
            egui::Slider::new(&mut settings.collision_weight, 0.0..=10.0).text("Collision Weight"),
        );
        ui.add(
            egui::Slider::new(&mut settings.alignment_weight, 0.0..=10.0).text("Alignment Weight"),
        );
        ui.add(
            egui::Slider::new(&mut settings.alignment_weight, 0.0..=10.0).text("Alignment Weight"),
        );
        ui.add(
            egui::Slider::new(&mut settings.cohesion_weight, 0.0..=10.0).text("Cohesion Weight"),
        );
        ui.add(egui::Slider::new(&mut settings.seek_weight, 0.0..=100.0).text("Follower: Weight"));
        ui.add(
            egui::Slider::new(&mut settings.seek_max_speed, 0.0..=2.0).text("Follower: Max Speed"),
        );
        ui.add(
            egui::Slider::new(&mut settings.seek_max_force, 0.0..=2.0).text("Follower: Max Force"),
        );
        ui.add(
            egui::Slider::new(&mut settings.controller_turn_speed, 0.0..=10.0)
                .text("Controller Turn Speed"),
        );
        ui.add(
            egui::Slider::new(&mut settings.controller_acceleration, 0.0..=1000.0)
                .text("Controller Acceleration"),
        );
        ui.add(
            egui::Slider::new(&mut settings.controller_dampening, 0.0..=1000.0)
                .text("Controller Dampening"),
        );
        ui.add(
            egui::Slider::new(&mut settings.controller_max_speed, 0.0..=1000.0)
                .text("Controller Max Speed"),
        );

        ui.add(
            egui::Slider::new(&mut settings.max_hover_distance, 0.0..=1000.0)
                .text("Max Hover Distance"),
        );
        ui.add(
            egui::Slider::new(&mut settings.capture_time, 0.0..=10.0)
                .text("Time to Capture (secs)"),
        );

        ui.add(
            egui::Slider::new(&mut settings.projectile_speed, 0.0..=8000.0)
                .text("Projectile: Speed"),
        );
        ui.add(
            egui::Slider::new(&mut settings.projectile_despawn_distance, 0.0..=8000.0)
                .text("Projectile: Despawn Distance"),
        );
        ui.add(
            egui::Slider::new(&mut settings.projectile_cooldown, 0.0..=1.0)
                .text("Projectile: Cooldown (secs)"),
        );

        ui.add(
            egui::Slider::new(&mut settings.player_damage_amount, 0..=8000)
                .text("Player: Damage Amount"),
        );
        ui.add(
            egui::Slider::new(&mut settings.player_damage_cooldown, 0.0..=1.0)
                .text("Player: Damage Cooldown (secs)"),
        );

        ui.add(
            egui::Slider::new(&mut settings.player_max_health, 0..=1000).text("Player: Max Health"),
        );
        ui.add(
            egui::Slider::new(&mut settings.player_score_follower_multiplier, 0.0..=3.0)
                .text("Player: Score Follower Multiplier"),
        );
        ui.add(
            egui::Slider::new(&mut settings.player_damage_follower_multiplier, 0.0..=3.0)
                .text("Player: Damage Follower Multiplier"),
        );
    });
}
