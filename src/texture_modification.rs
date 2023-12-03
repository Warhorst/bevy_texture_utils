use bevy_render::prelude::*;

pub type PixelBytes = [u8; 4];

/// Modify the data of a texture with a given pixel mapper.
/// TODO: Currently only works with 4-byte-pixel-images, will crash if something else is provided.
pub fn modify_texture(
    texture: &mut Image,
    pixel_mapper: impl Fn(PixelBytes) -> PixelBytes
) {
    let data = &mut texture.data;
    let width = texture.texture_descriptor.size.width as usize;
    let height = texture.texture_descriptor.size.height as usize;

    for x in 0..width {
        for y in 0..height {
            let index = width * 4 * y + x * 4;

            let pixel = [
                data[index],
                data[index + 1],
                data[index + 2],
                data[index + 3],
            ];

            let new_pixel = pixel_mapper(pixel);

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
    pixel_mapper: impl Fn(PixelBytes) -> PixelBytes
) -> Image {
    let mut new_image = texture.clone();
    modify_texture(&mut new_image, pixel_mapper);

    new_image
}

#[cfg(test)]
mod tests {
    use bevy_render::prelude::*;
    use bevy_render::render_resource::TextureFormat;
    use crate::test_utils::create_image;
    use crate::texture_modification::{map_to_new_texture, modify_texture};

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
        modify_texture(&mut red_blue, |pixels| if pixels == Color::BLUE.as_rgba_u8() {
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
        let new_texture = map_to_new_texture(&red_blue, |pixels| if pixels == Color::BLUE.as_rgba_u8() {
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
}