use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};

use super::settings::SettingsResource;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin);
        app.add_systems(Update, render_settings_gui);
    }
}

pub fn render_settings_gui(mut settings: ResMut<SettingsResource>, mut contexts: EguiContexts) {
    egui::Window::new("SettingsResource").show(contexts.ctx_mut(), |ui| {
        ui.style_mut().spacing.slider_width = 300.0;

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
    });
}
