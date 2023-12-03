use std::collections::HashSet;

use bevy::asset::RecursiveDependencyLoadState;
use bevy::prelude::*;

use self::icons::IconSheetAsset;

use super::icons::IconSheetResource;
use super::states::GameState;

pub mod icons;

pub struct GameAssetPlugin;

impl Plugin for GameAssetPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<icons::IconSheetAsset>();
        app.init_asset_loader::<icons::IconSheetLoader>();
        app.add_systems(OnEnter(GameState::Init), load_assets_system);
        app.add_systems(
            Update,
            update_loading_system.run_if(in_state(GameState::AssetsLoading)),
        );
        app.add_systems(OnEnter(GameState::AssetsLoaded), assets_loaded_system);
    }
}

#[derive(Resource, Debug)]
pub struct PendingAssets(HashSet<UntypedHandle>);

fn load_assets_system(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut state: ResMut<NextState<GameState>>,
) {
    let icons: Handle<IconSheetAsset> = server.load("icons.icon.json");
    commands.insert_resource(IconSheetResource(icons.clone()));
    commands.insert_resource(PendingAssets(HashSet::from_iter(vec![icons.untyped()])));
    state.set(GameState::AssetsLoading);
}

fn update_loading_system(
    mut pending: ResMut<PendingAssets>,
    server: Res<AssetServer>,
    mut state: ResMut<NextState<GameState>>,
) {
    let mut errors = Vec::new();
    // info!("pending.0 -> {:?}", pending);

    pending.0.retain(|pending| {
        let path = server.get_path(pending.id());
        // println!("path -> {:?}", path);
        let states = server.get_load_states(pending.id());
        // info!("check loading: {:?} -> {:?}", pending, states);
        // println!("states -> {:?}", states);
        states.map_or(true, |(_, _, state)| {
            if state == RecursiveDependencyLoadState::Loaded {
                info!("Successfully loaded asset: {:?}", path);
                false
            } else if state == RecursiveDependencyLoadState::Failed {
                errors.push(format!("Failed loading asset: {:?}", path));
                false
            } else {
                true
            }
        })
    });

    if !errors.is_empty() {
        error!("Error loading assets:\n{}", errors.join("\n"));
    }

    if pending.0.is_empty() {
        state.set(GameState::AssetsLoaded);
    }
}

fn assets_loaded_system(
    server: Res<AssetServer>,
    images: Res<Assets<Image>>,
    mut state: ResMut<NextState<GameState>>,
) {
    images.iter().for_each(|(handle, image)| {
        info!(
            "Loaded image: {:?} ({}x{})",
            server.get_path(handle),
            image.width(),
            image.height(),
        );
    });

    state.set(GameState::GameLoading);
}
