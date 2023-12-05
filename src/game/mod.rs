use bevy::prelude::*;

mod assets;
mod camera;
mod debug;
mod icons;
mod render;
mod settings;
mod states;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(settings::SettingsResource::default());
        app.add_state::<states::GameState>();
        app.add_plugins((
            assets::GameAssetPlugin,
            icons::IconPlugin,
            render::RenderPlugin,
            camera::CameraPlugin,
        ));

        app.add_plugins(debug::DebugPlugin);
    }
}
