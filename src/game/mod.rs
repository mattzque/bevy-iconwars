use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;

mod assets;
mod camera;
mod debug;
mod hud;
mod icons;
mod render;
mod settings;
mod states;
mod world;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(settings::SettingsResource::default());
        app.add_state::<states::GameState>();
        app.add_plugins((
            assets::GameAssetPlugin,
            world::WorldPlugin,
            icons::IconPlugin,
            render::RenderPlugin,
            camera::CameraPlugin,
            hud::HudPlugin,
        ));

        app.add_plugins((debug::DebugPlugin, FrameTimeDiagnosticsPlugin));
    }
}
