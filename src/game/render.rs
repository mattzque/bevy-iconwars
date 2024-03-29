use bevy::prelude::*;

pub struct RenderPlugin;

const CLEAR_COLOR: &str = "#121418";

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(
            Color::hex(CLEAR_COLOR).expect("Invalid ClearColor!"),
        ));
        app.add_systems(Startup, spawn_render_globals);
    }
}

fn spawn_render_globals(mut commands: Commands) {
    commands.insert_resource(Msaa::Sample4);
}
