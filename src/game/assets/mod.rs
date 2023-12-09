use std::collections::HashSet;

use bevy::asset::RecursiveDependencyLoadState;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::render::texture::{ImageFilterMode, ImageSampler, ImageSamplerDescriptor};

use self::icons::IconSheetAsset;

use super::audio::AudioFileResource;
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
    let mut pending = HashSet::new();

    let icons: Handle<IconSheetAsset> = server.load("icons.icon.json");
    pending.insert(icons.clone().untyped());

    let font_title: Handle<Font> = server.load("fonts/GasoekOne-Regular.ttf");
    let font_text: Handle<Font> = server.load("fonts/DMSans-Black.ttf");
    let font_text2: Handle<Font> = server.load("fonts/DMSans-Regular.ttf");
    pending.insert(font_title.clone().untyped());
    pending.insert(font_text.clone().untyped());
    pending.insert(font_text2.clone().untyped());

    let music: Vec<Handle<AudioSource>> = vec![
        server.load("music/track2.ogg"),
        server.load("music/track3.ogg"),
        server.load("music/track4.ogg"),
    ];
    let shoot: Handle<AudioSource> = server.load("sfx/shoot_01.ogg");
    let hit: Handle<AudioSource> = server.load("sfx/weird_03.ogg");
    let capture: Handle<AudioSource> = server.load("sfx/misc_05.ogg");
    let damage: Handle<AudioSource> = server.load("sfx/misc_02.ogg");

    pending.extend(music.iter().map(|handle| handle.clone().untyped()));
    pending.insert(shoot.clone().untyped());
    pending.insert(hit.clone().untyped());
    pending.insert(capture.clone().untyped());
    pending.insert(damage.clone().untyped());

    commands.insert_resource(IconSheetResource {
        handle: icons.clone(),
        texture_array: None,
    });
    commands.insert_resource(FontResource {
        title: font_title.clone(),
        text: font_text.clone(),
        text2: font_text2.clone(),
    });
    commands.insert_resource(AudioFileResource {
        music,
        shoot,
        hit,
        capture,
        damage,
    });
    commands.insert_resource(PendingAssets(pending));
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
