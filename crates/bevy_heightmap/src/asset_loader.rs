use bevy::{
    asset::{AssetLoader, LoadContext, io::Reader},
    ecs::prelude::{FromWorld, World},
    image::{
        CompressedImageFormats, Image, ImageFormat, ImageFormatSetting, ImageLoaderSettings,
        ImageType, IntoDynamicImageError, TextureError,
    },
    mesh::Mesh,
    reflect::TypePath,
    render::renderer::RenderDevice,
};

use thiserror::Error;

use crate::HeightMap;
use crate::image::ImageBufferHeightMap;

/// Loader for images that can be read by the `image` crate.
#[derive(Clone, TypePath)]
pub struct HeightMapLoader {
    supported_compressed_formats: CompressedImageFormats,
}
impl HeightMapLoader {
    pub const EXTENSIONS: &'static [&'static str] = &["hmp.png"];
}

impl AssetLoader for HeightMapLoader {
    type Asset = Mesh;
    type Settings = ImageLoaderSettings;
    type Error = HeightMapLoaderError;
    async fn load(
        &self,
        reader: &mut dyn Reader,
        settings: &ImageLoaderSettings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Mesh, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let image_type = match settings.format {
            ImageFormatSetting::FromExtension => {
                // use the file extension for the image type
                let ext = load_context
                    .path()
                    .path()
                    .extension()
                    .unwrap()
                    .to_str()
                    .unwrap();
                ImageType::Extension(ext)
            }
            ImageFormatSetting::Format(format) => ImageType::Format(format),
            ImageFormatSetting::Guess => {
                let format = image::guess_format(&bytes).map_err(|err| HeightMapFileError {
                    error: err.into(),
                    path: format!("{}", load_context.path().path().display()),
                })?;
                ImageType::Format(ImageFormat::from_image_crate_format(format).ok_or_else(
                    || HeightMapFileError {
                        error: TextureError::UnsupportedTextureFormat(format!("{format:?}")),
                        path: format!("{}", load_context.path().path().display()),
                    },
                )?)
            }
        };
        let image: Image = Image::from_buffer(
            &bytes,
            image_type,
            self.supported_compressed_formats,
            settings.is_srgb,
            settings.sampler.clone(),
            settings.asset_usage,
        )
        .map_err(|err| HeightMapFileError {
            error: err,
            path: format!("{}", load_context.path().path().display()),
        })?;
        let image_heightmap = ImageBufferHeightMap::try_from_image(image.clone())?;
        Ok(image_heightmap.build_mesh(image.size()))
    }

    fn extensions(&self) -> &[&str] {
        &["png"]
    }
}

impl FromWorld for HeightMapLoader {
    fn from_world(world: &mut World) -> Self {
        let supported_compressed_formats = match world.get_resource::<RenderDevice>() {
            Some(render_device) => CompressedImageFormats::from_features(render_device.features()),

            None => CompressedImageFormats::NONE,
        };
        Self {
            supported_compressed_formats,
        }
    }
}

/// An error that occurs when loading a texture from a file.
#[derive(Error, Debug)]
pub struct HeightMapFileError {
    error: TextureError,
    path: String,
}
impl std::fmt::Display for HeightMapFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "Error reading image file {}: {}, this is an error in `bevy_render`.",
            self.path, self.error
        )
    }
}

#[non_exhaustive]
#[derive(Debug, Error)]
pub enum HeightMapLoaderError {
    #[error("Could load shader: {0}")]
    Io(#[from] std::io::Error),
    #[error("Could not load texture file: {0}")]
    FileTexture(#[from] HeightMapFileError),
    #[error("Error converting to dynamic image: {0}")]
    IntoDynamicImageError(#[from] IntoDynamicImageError),
    #[error("Unsupported image type")]
    UnsupportedImageType,
}
