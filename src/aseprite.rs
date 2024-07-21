use std::io::Cursor;

use asefile::AsepriteFile;
use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext},
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
        texture::Image,
    },
};

pub struct AsepriteLoader;

impl AssetLoader for AsepriteLoader {
    type Asset = Image;
    type Settings = ();
    type Error = anyhow::Error;
    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        _settings: &'a (),
        _load_context: &'a mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let aseprite = AsepriteFile::read(Cursor::new(bytes))?;
        let image = aseprite.frame(0).image();
        Ok(Image::new(
            Extent3d {
                width: image.width(),
                height: image.height(),
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            image.into_vec(),
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::all(),
        ))
    }

    fn extensions(&self) -> &[&str] {
        &["aseprite"]
    }
}
