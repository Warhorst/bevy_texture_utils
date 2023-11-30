use std::collections::HashMap;
use bevy_asset::prelude::*;
use bevy_render::prelude::*;
use bevy_render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use pad::{p, Position};

const BYTES_PER_PIXEL: usize = 4;
const TILE_DIMENSION: usize = 2;

// TODO: tile dimension must be a parameter
// TODO: Make sure every image uses the same TextureFormat

/// Combine multiple given textures to a single one, forming
/// a tile map texture.
pub fn create_tile_map_texture(
    images: &mut Assets<Image>,
    positions_and_textures: impl IntoIterator<Item=(Position, Handle<Image>)>,
) -> Result<Handle<Image>, &'static str> {
    let position_texture_map = positions_and_textures
        .into_iter()
        .collect::<HashMap<_, _>>();

    let max_x = get_max_x(&position_texture_map)?;
    let min_x = get_min_x(&position_texture_map)?;
    let max_y = get_max_y(&position_texture_map)?;
    let min_y = get_min_y(&position_texture_map)?;

    let width = (max_x - min_x) + 1;
    let height = (max_y - min_y) + 1;

    let mut data = vec![0u8; (width * TILE_DIMENSION * BYTES_PER_PIXEL) * (height * TILE_DIMENSION)];

    for y in (min_y..=max_y).rev() {
        for x in min_x..=max_x {
            let absolute_pos = p!(x, y);
            let relative_pos = p!(x - min_x, max_y - y);

            let image = match position_texture_map.get(&absolute_pos) {
                Some(handle) => match images.get(handle) {
                    Some(image) => image,
                    None => return Err("Not every image was already loaded")
                },
                None => continue,
            };

            let image_data = &image.data;

            add_data_from_tile_image_at_position(width, &mut data, &relative_pos, image_data);
        }
    }

    let tiles_texture = create_image_from_data(width, height, data);
    Ok(images.add(tiles_texture))
}

fn get_max_x(position_texture_map: &HashMap<Position, Handle<Image>>) -> Result<usize, &'static str> {
    let max_opt = position_texture_map
        .keys()
        .map(|pos| pos.x)
        .max();

    let max = match max_opt {
        Some(max) => max,
        None => return Err("No tiles were provided!")
    };

    Ok(max as usize)
}

fn get_min_x(position_texture_map: &HashMap<Position, Handle<Image>>) -> Result<usize, &'static str> {
    let min_opt = position_texture_map
        .keys()
        .map(|pos| pos.x)
        .min();

    let min = match min_opt {
        Some(min) => min,
        None => return Err("No tiles were provided!")
    };

    Ok(min as usize)
}

fn get_max_y(position_texture_map: &HashMap<Position, Handle<Image>>) -> Result<usize, &'static str> {
    let max_opt = position_texture_map
        .keys()
        .map(|pos| pos.y)
        .max();

    let max = match max_opt {
        Some(max) => max,
        None => return Err("No tiles were provided!")
    };

    Ok(max as usize)
}

fn get_min_y(position_texture_map: &HashMap<Position, Handle<Image>>) -> Result<usize, &'static str> {
    let min_opt = position_texture_map
        .keys()
        .map(|pos| pos.y)
        .min();

    let min = match min_opt {
        Some(min) => min,
        None => return Err("No tiles were provided!")
    };

    Ok(min as usize)
}

fn add_data_from_tile_image_at_position(width: usize, data: &mut [u8], pos: &Position, image_data: &[u8]) {
    for y in 0..TILE_DIMENSION {
        for x in 0..TILE_DIMENSION {
            for i in 0..BYTES_PER_PIXEL {
                let image_index = y * TILE_DIMENSION * BYTES_PER_PIXEL + x * BYTES_PER_PIXEL + i;

                let tiles_texture_index =
                    (width * TILE_DIMENSION * BYTES_PER_PIXEL) * (pos.y as usize * TILE_DIMENSION) // move to the first row the tile is contained in
                        + (pos.x as usize * TILE_DIMENSION * BYTES_PER_PIXEL) // than move to the first pixel of the tile
                        + (TILE_DIMENSION * BYTES_PER_PIXEL * width * y) // than move to the current row of the tile
                        + x * BYTES_PER_PIXEL // than move to the current pixel block
                        + i; // than finally move to the current pixel

                data[tiles_texture_index] = image_data[image_index];
            }
        }
    }
}

fn create_image_from_data(max_x: usize, max_y: usize, data: Vec<u8>) -> Image {
    Image::new(
        Extent3d {
            width: (max_x * TILE_DIMENSION) as u32,
            height: (max_y * TILE_DIMENSION) as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
    )
}

#[cfg(test)]
mod tests {
    use pad::p;
    use bevy_asset::prelude::*;
    use bevy_render::prelude::*;
    use bevy_render::render_resource::{Extent3d, TextureDimension, TextureFormat};
    use crate::create_tile_map_texture;

    #[test]
    fn create_tile_map_texture_works() {
        // arrange
        let mut images = Assets::<Image>::default();
        let red = images.add(create_image(
            (2, 2),
            TextureFormat::Rgba8UnormSrgb,
            [
                Color::RED, Color::RED,
                Color::RED,Color::RED
            ]
        ));
        let green = images.add(create_image(
            (2, 2),
            TextureFormat::Rgba8UnormSrgb,
            [
                Color::GREEN, Color::GREEN,
                Color::GREEN,Color::GREEN
            ]
        ));

        // act
        let image_result = create_tile_map_texture(
            &mut images,
            [
                (p!(0, 0), red.clone()),
                (p!(1, 0), green.clone()),
                (p!(0, 1), green),
                (p!(1, 1), red),
            ]
        );

        // assert
        assert!(image_result.is_ok());
        let new_image_handle = image_result.unwrap();

        let expected_image = create_image(
            (4, 4),
            TextureFormat::Rgba8UnormSrgb,
            [
                Color::GREEN, Color::GREEN, Color::RED, Color::RED,
                Color::GREEN, Color::GREEN, Color::RED, Color::RED,
                Color::RED, Color::RED, Color::GREEN, Color::GREEN,
                Color::RED, Color::RED, Color::GREEN, Color::GREEN,
            ]
        );

        assert_eq!(
            &images.get(new_image_handle).unwrap().data,
            &expected_image.data
        );
    }

    /// Create an image with the given dimension, texture format and colors for each pixel.
    /// Dimension and given pixel must match in size. The first pixel is top left of the image
    /// and the last one is bottom right.
    fn create_image(
        (width, height): (usize, usize),
        texture_format: TextureFormat,
        pixel_colors: impl IntoIterator<Item=Color>
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
}