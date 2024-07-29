use std::io::Cursor;

use asefile::AsepriteFile;
use bevy::{
    asset::{io::Reader, AssetLoader, AsyncReadExt, LoadContext},
    prelude::*,
    render::{
        render_asset::RenderAssetUsages,
        render_resource::{Extent3d, TextureDimension, TextureFormat},
    },
};

pub struct AsepriteImageLoader;

impl AssetLoader for AsepriteImageLoader {
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

pub fn animations(
    mut query: Query<(&mut Handle<Image>, &mut Animation)>,
    time: Res<Time>,
    assets: Res<Assets<AnimationData>>,
) {
    for (mut tex, mut ani) in &mut query {
        ani.timer -= time.delta_seconds();
        if ani.timer < 0. {
            let data = assets.get(&ani.data).unwrap();
            if ani.index as usize == data.frames.len() {
                continue;
            }
            *tex = data.frames[ani.index as usize].0.clone();
            ani.timer += data.frames[ani.index as usize].1;
            ani.index += 1;
            if ani.repeat & (ani.index as usize == data.frames.len()) {
                ani.index = 0;
            }
        }
    }
}

#[derive(Component)]
pub struct Animation {
    data: Handle<AnimationData>,
    index: i32,
    timer: f32,
    repeat: bool,
}

impl Animation {
    pub fn new(data: Handle<AnimationData>, repeat: bool) -> Self {
        Self {
            data,
            index: 0,
            timer: 0.,
            repeat,
        }
    }
}

#[derive(Asset, TypePath)]
pub struct AnimationData {
    pub frames: Vec<(Handle<Image>, f32)>,
}

pub struct AsepriteAniLoader;

impl AssetLoader for AsepriteAniLoader {
    type Asset = AnimationData;
    type Settings = ();
    type Error = anyhow::Error;
    async fn load<'a>(
        &'a self,
        reader: &'a mut Reader<'_>,
        _settings: &'a (),
        load_context: &'a mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let aseprite = AsepriteFile::read(Cursor::new(bytes))?;
        let mut frames = Vec::new();
        for i in 0..aseprite.num_frames() {
            let frame = aseprite.frame(i);
            let image = frame.image();
            let handle = load_context.labeled_asset_scope(i.to_string(), move |_| {
                Image::new(
                    Extent3d {
                        width: image.width(),
                        height: image.height(),
                        depth_or_array_layers: 1,
                    },
                    TextureDimension::D2,
                    image.into_vec(),
                    TextureFormat::Rgba8UnormSrgb,
                    RenderAssetUsages::all(),
                )
            });
            let duration = frame.duration() as f32 * 0.001;
            frames.push((handle, duration));
        }
        Ok(AnimationData { frames })
    }

    fn extensions(&self) -> &[&str] {
        &["aseprite"]
    }
}
