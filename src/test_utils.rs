use bevy_render::prelude::*;
use bevy_render::render_resource::{Extent3d, TextureDimension, TextureFormat};

/// Create an image with the given dimension, texture format and colors for each pixel.
/// Dimension and given pixel must match in size. The first pixel is top left of the image
/// and the last one is bottom right.
pub fn create_image(
    (width, height): (usize, usize),
    texture_format: TextureFormat,
    pixel_colors: impl IntoIterator<Item=Color>,
) -> Image {
    let data = pixel_colors
        .into_iter()
        .flat_map(|c| c.as_rgba_u8())
        .collect::<Vec<_>>();

    if data.len() / 4 != width * height {
        panic!("Given data and dimension don't match!")
    }

    Image::new(
        Extent3d {
            width: width as u32,
            height: height as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        texture_format,
    )
}