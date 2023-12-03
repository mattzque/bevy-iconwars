use bevy::prelude::*;

mod assets;
mod camera;
mod icons;
mod render;
mod states;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<states::GameState>();
        app.add_plugins((
            assets::GameAssetPlugin,
            icons::IconPlugin,
            render::RenderPlugin,
            camera::CameraPlugin,
        ));
    }
}
