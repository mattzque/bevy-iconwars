use std::collections::HashSet;

use bevy::asset::RecursiveDependencyLoadState;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::render::texture::{ImageFilterMode, ImageSampler, ImageSamplerDescriptor};

use self::icons::IconSheetAsset;

use super::hud::FontResource;
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
    let font_title: Handle<Font> = server.load("fonts/GasoekOne-Regular.ttf");
    let font_text: Handle<Font> = server.load("fonts/DMSans-Black.ttf");
    let icons: Handle<IconSheetAsset> = server.load("icons.icon.json");
    commands.insert_resource(IconSheetResource {
        handle: icons.clone(),
        texture_array: None,
    });
    commands.insert_resource(FontResource {
        title: font_title.clone(),
        text: font_text.clone(),
    });
    commands.insert_resource(PendingAssets(HashSet::from_iter(vec![
        icons.untyped(),
        font_title.untyped(),
        font_text.untyped(),
    ])));
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
    mut state: ResMut<NextState<GameState>>,
    mut resource: ResMut<IconSheetResource>,
    assets: Res<Assets<IconSheetAsset>>,
    mut images: ResMut<Assets<Image>>,
) {
    let IconSheetAsset(sheets) = assets.get(&resource.handle).unwrap();
    let (width, height) = sheets
        .first()
        .map(|sheet| (sheet.width, sheet.height))
        .unwrap();
    let textures = sheets
        .iter()
        .flat_map(|sheet| images.get(sheet.handle.id()))
        .collect::<Vec<&Image>>();

    let mut data = Vec::new();
    textures.iter().for_each(|texture| {
        // println!("texture format: {:?}", texture.texture_descriptor);
        data.extend_from_slice(&texture.data);
    });

    // create array texture:
    let mut texture_array = Image::new(
        Extent3d {
            width: width as u32,
            height: height as u32,
            depth_or_array_layers: textures.len() as u32,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        // TextureFormat::Rgba8Unorm,
        // TextureFormat::// Rgba8UnormSrgb,
    );
    texture_array.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        mag_filter: ImageFilterMode::Nearest,
        min_filter: ImageFilterMode::Nearest,
        mipmap_filter: ImageFilterMode::Nearest,
        // lod_min_clamp: (),
        // lod_max_clamp: (),
        ..Default::default()
    });
    let texture_array_handle = images.add(texture_array);
    resource.texture_array = Some(texture_array_handle);

    state.set(GameState::GameLoading);
}
