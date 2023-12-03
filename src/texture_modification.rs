use bevy_render::prelude::*;
use bevy_render::render_resource::Texture;

pub type PixelBytes = [u8; 4];

/// Modify the data of a texture with a given pixel mapper. The mapper takes the x and y coordinates
/// of the pixel and also the pixel bytes at these coordinates.
/// TODO: Currently only works with 4-byte-pixel-images, will crash if something else is provided.
pub fn modify_texture(
    texture: &mut Image,
    pixel_mapper: impl Fn(usize, usize, PixelBytes) -> PixelBytes,
) {
    let width = texture.width() as usize;
    let height = texture.height() as usize;
    let data = &mut texture.data;

    for x in 0..width {
        for y in 0..height {
            let index = width * 4 * y + x * 4;

            let pixel = [
                data[index],
                data[index + 1],
                data[index + 2],
                data[index + 3],
            ];

            let new_pixel = pixel_mapper(x, y, pixel);

            data[index] = new_pixel[0];
            data[index + 1] = new_pixel[1];
            data[index + 2] = new_pixel[2];
            data[index + 3] = new_pixel[3];
        }
    }
}

/// Takes a texture and a pixel mapper and creates a new texture from if.
/// TODO: Currently only works with 4-byte-pixel-images, will crash if something else is provided.
pub fn map_to_new_texture(
    texture: &Image,
    pixel_mapper: impl Fn(usize, usize, PixelBytes) -> PixelBytes,
) -> Image {
    let mut new_image = texture.clone();
    modify_texture(&mut new_image, pixel_mapper);

    new_image
}

/// Provides a pixel mapper to replace the pixels of the original texture
/// with ones from another texture. Also takes a pixel filter to tell
/// if the pixel should be replaced by the pixel from the other texture.
pub fn map_to_texture_pixels<'a>(
    texture: &'a Image,
    pixel_filter: fn(&PixelBytes) -> bool,
) -> impl Fn(usize, usize, PixelBytes) -> PixelBytes + 'a {
    move |x, y, pixel| if !pixel_filter(&pixel) {
        pixel
    } else {
        let width = texture.width() as usize;
        let x = x % width;
        let y = y % texture.height() as usize;
        let index = width * 4 * y + x * 4;
        [
            texture.data[index],
            texture.data[index + 1],
            texture.data[index + 2],
            texture.data[index + 3]
        ]
    }
}

#[cfg(test)]
mod tests {
    use bevy_render::prelude::*;
    use bevy_render::render_resource::TextureFormat;
    use crate::test_utils::create_image;
    use crate::texture_modification::{map_to_new_texture, map_to_texture_pixels, modify_texture};

    #[test]
    fn modify_texture_works() {
        // arrange
        let mut red_blue = create_image(
            (2, 2),
            TextureFormat::Rgba8UnormSrgb,
            [
                Color::RED, Color::BLUE,
                Color::BLUE, Color::RED
            ],
        );

        // act
        modify_texture(&mut red_blue, |_, _, pixels| if pixels == Color::BLUE.as_rgba_u8() {
            Color::GREEN.as_rgba_u8()
        } else {
            pixels
        });

        // assert
        let expected = create_image(
            (2, 2),
            TextureFormat::Rgba8UnormSrgb,
            [
                Color::RED, Color::GREEN,
                Color::GREEN, Color::RED
            ],
        );

        assert_eq!(expected.data, red_blue.data, "The red-blue image should now be red-green, but wasn't.");
    }

    #[test]
    fn map_to_new_texture_works() {
        // arrange
        let red_blue = create_image(
            (2, 2),
            TextureFormat::Rgba8UnormSrgb,
            [
                Color::RED, Color::BLUE,
                Color::BLUE, Color::RED
            ],
        );

        // act
        let new_texture = map_to_new_texture(&red_blue, |_, _, pixels| if pixels == Color::BLUE.as_rgba_u8() {
            Color::GREEN.as_rgba_u8()
        } else {
            pixels
        });

        // assert
        let expected = create_image(
            (2, 2),
            TextureFormat::Rgba8UnormSrgb,
            [
                Color::RED, Color::GREEN,
                Color::GREEN, Color::RED
            ],
        );

        assert_eq!(expected.data, new_texture.data, "The red-blue image should now be mapped to a red-green one, but wasn't.");
    }

    /// Check if the convenience method to apply a texture on another one works. The
    /// texture to map from is intentionally smaller to check if the mapper can
    /// handle this.
    #[test]
    fn modify_texture_with_other_texture_works() {
        // arrange
        let mut red_blue = create_image(
            (4, 4),
            TextureFormat::Rgba8UnormSrgb,
            [
                Color::RED, Color::BLUE,Color::RED, Color::BLUE,
                Color::BLUE, Color::RED, Color::BLUE, Color::RED,
                Color::RED, Color::BLUE,Color::RED, Color::BLUE,
                Color::BLUE, Color::RED, Color::BLUE, Color::RED,
            ],
        );

        let yellow_green = create_image(
            (2, 2),
            TextureFormat::Rgba8UnormSrgb,
            [
                Color::YELLOW, Color::GREEN,
                Color::YELLOW, Color::GREEN
            ],
        );

        // act
        modify_texture(&mut red_blue, map_to_texture_pixels(&yellow_green, |pixels| pixels == &Color::BLUE.as_rgba_u8()));

        // assert
        let expected = create_image(
            (4, 4),
            TextureFormat::Rgba8UnormSrgb,
            [
                Color::RED, Color::GREEN,Color::RED, Color::GREEN,
                Color::YELLOW, Color::RED, Color::YELLOW, Color::RED,
                Color::RED, Color::GREEN,Color::RED, Color::GREEN,
                Color::YELLOW, Color::RED, Color::YELLOW, Color::RED,
            ],
        );

        assert_eq!(expected.data, red_blue.data, "The red-blue texture should now be red-green-yellow, but wasn't.")
    }
}