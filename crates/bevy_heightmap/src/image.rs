use bevy::{
    image::Image,
    math::{UVec2, Vec2},
};
use image::{DynamicImage, ImageBuffer, Pixel, Rgba};

use crate::{HeightMap, asset_loader::HeightMapLoaderError};

static DEFAULT_SMOOTH_RADIUS: i32 = 3;

pub struct ImageBufferHeightMap<P: Pixel, Container> {
    pub buffer: ImageBuffer<P, Container>,
    pub pixel_scale: Vec2,
    pub bounds: UVec2,
}
impl<P: Pixel> ImageBufferHeightMap<P, Vec<P::Subpixel>>
where
    P::Subpixel: bytemuck::Pod + Into<f32>,
{
    pub fn from_bytes(size: UVec2, buffer: &[u8]) -> Self {
        let bounds = size - UVec2::ONE;
        let pixel_scale = bounds.as_vec2();
        let buffer =
            ImageBuffer::from_raw(size.x, size.y, bytemuck::cast_slice(buffer).to_owned()).unwrap();
        Self {
            buffer,
            pixel_scale,
            bounds,
        }
    }
}
impl ImageBufferHeightMap<Rgba<u8>, Vec<u8>> {
    pub fn try_from_image(image: Image) -> Result<Self, HeightMapLoaderError> {
        let size = image.size();
        let bounds = size - UVec2::ONE;
        let pixel_scale = bounds.as_vec2();
        let DynamicImage::ImageRgba8(buffer) = image.try_into_dynamic()? else {
            return Err(HeightMapLoaderError::UnsupportedImageType);
        };
        Ok(Self {
            buffer,
            pixel_scale,
            bounds,
        })
    }
}

fn calculate_smoothed_height<P>(
    buffer: &ImageBuffer<P, Vec<P::Subpixel>>,
    bounds: UVec2,
    center: UVec2,
    radius: i32,
) -> f32
where
    P: Pixel,
    P::Subpixel: Into<f32> + Copy,
{
    let mut total_value = 0.0;
    let mut num_samples = 0;

    for dy in -radius..=radius {
        for dx in -radius..=radius {
            let sample_x = (center.x as i32 + dx).clamp(0, bounds.x as i32) as u32;
            let sample_y = (center.y as i32 + dy).clamp(0, bounds.y as i32) as u32;
            let pixel_value = buffer.get_pixel(sample_x, bounds.y - sample_y).channels()[0];
            total_value += pixel_value.into();
            num_samples += 1;
        }
    }
    total_value / num_samples as f32
}

impl HeightMap for ImageBufferHeightMap<Rgba<f32>, Vec<f32>> {
    fn h(&self, p: Vec2) -> f32 {
        let center_xy = (self.pixel_scale * (p + Vec2::ONE / 2.)).as_uvec2();
        calculate_smoothed_height(&self.buffer, self.bounds, center_xy, DEFAULT_SMOOTH_RADIUS)
    }
}

impl HeightMap for ImageBufferHeightMap<Rgba<u8>, Vec<u8>> {
    fn h(&self, p: Vec2) -> f32 {
        let center_xy = (self.pixel_scale * (p + Vec2::ONE / 2.)).as_uvec2();
        let smoothed_value =
            calculate_smoothed_height(&self.buffer, self.bounds, center_xy, DEFAULT_SMOOTH_RADIUS);
        smoothed_value / 255.0
    }
}
