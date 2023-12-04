use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext},
    prelude::*,
    reflect::TypeUuid,
    utils::BoxedFuture,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Icon {
    pub name: String,
    pub x: usize,
    pub y: usize,
}

/// IconSheetFile as represented in the JSON file
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IconSheetFile {
    pub filename: String,
    pub width: usize,
    pub height: usize,
    pub tile_width: usize,
    pub tile_height: usize,
    pub tiles: Vec<Icon>,
}

/// IconSheet with the texture handle dependency
#[derive(Debug)]
pub struct IconSheet {
    pub handle: Handle<Image>,
    pub width: usize,
    pub height: usize,
    pub tile_width: usize,
    pub tile_height: usize,
    pub tiles: Vec<Icon>,
}

#[derive(Asset, Debug, TypeUuid, TypePath)]
#[uuid = "bb63465d-326f-47b6-8c49-008128f3e863"]
pub struct IconSheetAsset(pub Vec<IconSheet>);

#[derive(Default)]
pub struct IconSheetLoader;

#[derive(thiserror::Error, Debug)]
pub enum AssetError {
    #[error("Asset Error: {0}")]
    AnyError(#[from] anyhow::Error),

    #[error("Asset IO Error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Asset JSON Error: {0}")]
    YamlError(#[from] serde_json::Error),
}

impl AssetLoader for IconSheetLoader {
    type Asset = IconSheetAsset;
    type Settings = ();
    type Error = AssetError;

    fn load<'a>(
        &'a self,
        reader: &'a mut Reader,
        _settings: &'a Self::Settings,
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let sheets = serde_json::from_slice::<Vec<IconSheetFile>>(&bytes)?
                .into_iter()
                .map(|sheet| {
                    // load sheet as a dependency
                    info!("Load icon sheet: {:?}", sheet.filename);
                    let handle: Handle<Image> = load_context.load(&sheet.filename);
                    IconSheet {
                        handle,
                        width: sheet.width,
                        height: sheet.height,
                        tile_width: sheet.tile_width,
                        tile_height: sheet.tile_height,
                        tiles: sheet.tiles,
                    }
                })
                .collect();
            Ok(IconSheetAsset(sheets))
        })
    }

    fn extensions(&self) -> &[&str] {
        &["icon.json"]
    }
}
