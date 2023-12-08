use bevy::{
    app::{App, PluginGroup},
    asset::AssetMetaCheck,
    log::LogPlugin,
    window::{PresentMode, Window, WindowPlugin, WindowResolution},
    DefaultPlugins,
};
mod game;

fn main() {
    App::new()
        .insert_resource(AssetMetaCheck::Never)
        .add_plugins((DefaultPlugins
            .set(LogPlugin {
                filter: "info,wgpu_core=warn,wgpu_hal=warn,bevy_iconwars=debug".into(),
                level: bevy::log::Level::DEBUG,
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Icon Wars".to_string(),
                    present_mode: PresentMode::AutoVsync,
                    fit_canvas_to_parent: true,
                    resizable: true,
                    resolution: WindowResolution::new(1200.0, 1200.0),
                    ..Default::default()
                }),
                ..Default::default()
            }),))
        .add_plugins(game::GamePlugin)
        .run();
}
