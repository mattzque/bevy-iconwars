use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;

mod assets;
mod audio;
mod camera;
#[cfg(not(target_arch = "wasm32"))]
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
        app.init_state::<states::GameState>();
        app.add_plugins((
            assets::GameAssetPlugin,
            world::WorldPlugin,
            icons::IconPlugin,
            render::RenderPlugin,
            camera::CameraPlugin,
            hud::HudPlugin,
            audio::AudioPlugin,
        ));
        #[cfg(not(target_arch = "wasm32"))]
        app.add_plugins(debug::DebugPlugin);
        app.add_plugins(FrameTimeDiagnosticsPlugin);
    }
}
