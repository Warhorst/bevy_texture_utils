use std::collections::HashMap;
use bevy_asset::prelude::*;
use bevy_render::prelude::*;
use bevy_render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy_render::texture::TextureFormatPixelInfo;
use pad::{p, Position};

// TODO: Make sure every image uses the same TextureFormat

/// Creates tile map textures.
pub struct TileMapTextureCreator {
    /// The expected texture format of every image
    texture_format: TextureFormat,
    /// The amount of bytes each pixel of the given textures consists of.
    bytes_per_pixel: usize,
    /// The expected width of each tile texture
    tile_width: usize,
    /// The expected height of each tile texture
    tile_height: usize
}

impl TileMapTextureCreator {
    pub fn new(texture_format: TextureFormat, tile_width: usize, tile_height: usize) -> Self {
        Self { texture_format, bytes_per_pixel: texture_format.pixel_size(), tile_width, tile_height }
    }

    /// Combine multiple given textures to a single one, forming
    /// a tile map texture.
    pub fn create_tile_map_texture(
        &self,
        images: &mut Assets<Image>,
        positions_and_textures: impl IntoIterator<Item=(Position, Handle<Image>)>,
    ) -> Result<Handle<Image>, &'static str> {
        let position_texture_map = positions_and_textures
            .into_iter()
            .collect::<HashMap<_, _>>();

        let max_x = Self::get_max_x(&position_texture_map)?;
        let min_x = Self::get_min_x(&position_texture_map)?;
        let max_y = Self::get_max_y(&position_texture_map)?;
        let min_y = Self::get_min_y(&position_texture_map)?;

        let width = (max_x - min_x) + 1;
        let height = (max_y - min_y) + 1;

        let mut data = vec![0u8; (width * self.tile_width * self.bytes_per_pixel) * (height * self.tile_height)];

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

                self.add_data_from_tile_image_at_position(width, &mut data, &relative_pos, image_data);
            }
        }

        let tiles_texture = self.create_image_from_data(width, height, data);
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

    fn add_data_from_tile_image_at_position(&self, width: usize, data: &mut [u8], pos: &Position, image_data: &[u8]) {
        for y in 0..self.tile_height {
            for x in 0..self.tile_width {
                for i in 0..self.bytes_per_pixel {
                    let image_index = y * self.tile_height * self.bytes_per_pixel + x * self.bytes_per_pixel + i;

                    let tiles_texture_index =
                        (width * self.tile_width * self.bytes_per_pixel) * (pos.y as usize * self.tile_height) // move to the first row the tile is contained in
                            + (pos.x as usize * self.tile_width * self.bytes_per_pixel) // than move to the first pixel of the tile
                            + (self.tile_height * self.bytes_per_pixel * width * y) // than move to the current row of the tile
                            + x * self.bytes_per_pixel // than move to the current pixel block
                            + i; // than finally move to the current pixel

                    data[tiles_texture_index] = image_data[image_index];
                }
            }
        }
    }

    fn create_image_from_data(&self, max_x: usize, max_y: usize, data: Vec<u8>) -> Image {
        Image::new(
            Extent3d {
                width: (max_x * self.tile_width) as u32,
                height: (max_y * self.tile_height) as u32,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            data,
            self.texture_format,
        )
    }
}



#[cfg(test)]
mod tests {
    use pad::p;
    use bevy_asset::prelude::*;
    use bevy_render::prelude::*;
    use bevy_render::render_resource::{Extent3d, TextureDimension, TextureFormat};
    use crate::{TileMapTextureCreator};

    #[test]
    fn create_tile_map_texture_works() {
        // arrange
        let creator = TileMapTextureCreator::new(TextureFormat::Rgba8UnormSrgb, 2, 2);
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
        let image_result = creator.create_tile_map_texture(
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