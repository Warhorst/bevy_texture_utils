use bevy_asset::prelude::*;
use bevy_render::prelude::*;
use bevy_render::render_resource::{Extent3d, TextureDimension, TextureFormat};

/// The x, y and z offset of a texture. Tells
/// where to put the texture relative to (0, 0) and
/// on which layer.
#[derive(Copy, Clone)]
pub struct Offset {
    x: usize,
    y: usize,
    z: isize,
}

impl Offset {
    pub fn new(x: usize, y: usize, z: isize) -> Self {
        Self { x, y, z }
    }
}

// TODO find a better name
// TODO only works for 4 byte pixel images
pub fn mash_textures(
    images: &mut Assets<Image>,
    offsets_handles: impl IntoIterator<Item=(Offset, Handle<Image>)>,
) -> Result<Handle<Image>, String> {
    let offsets_textures_opt = offsets_handles
        .into_iter()
        .map(|(offset, handle)| images.get(handle).map(|t| (offset, t)))
        .collect::<Option<Vec<(Offset, &Image)>>>();

    let mut offsets_textures = match offsets_textures_opt {
        Some(ots) => ots,
        None => return Err("Some textures could not be retrieved. Maybe they aren't loaded yet".to_string())
    };

    offsets_textures.sort_by(|(offset_0, _), (offset_1, _)| offset_0.z.cmp(&offset_1.z));

    let image_width = offsets_textures
        .iter()
        .map(|(ofs, txt)| ofs.x + txt.width() as usize)
        .max()
        .ok_or("No texture handles were provided")?;

    let image_height = offsets_textures
        .iter()
        .map(|(ofs, txt)| ofs.y + txt.height() as usize)
        .max()
        .ok_or("No texture handles were provided")?;

    let mut image_data = vec![0; image_width * image_height * 4];

    for (offset, texture) in offsets_textures {
        let data = &texture.data;
        let part_width = texture.width() as usize;
        let part_height = texture.height() as usize;

        for y in 0..part_height {
            for x in 0..part_width {
                let mash_texture_index = image_width * 4 * (y + offset.y) + (x + offset.x) * 4;
                let part_texture_index = part_width * 4 * y + x * 4;

                image_data[mash_texture_index] = data[part_texture_index];
                image_data[mash_texture_index + 1] = data[part_texture_index + 1];
                image_data[mash_texture_index + 2] = data[part_texture_index + 2];
                image_data[mash_texture_index + 3] = data[part_texture_index + 3];
            }
        }
    }

    let image = Image::new(
        Extent3d {
            width: image_width as u32,
            height: image_height as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        image_data,
        TextureFormat::Rgba8UnormSrgb,
    );

    Ok(images.add(image))
}

#[cfg(test)]
mod tests {
    use bevy_asset::prelude::*;
    use bevy_render::prelude::*;
    use bevy_render::render_resource::TextureFormat;

    use crate::test_utils::create_image;
    use crate::texture_mashup::{mash_textures, Offset};

    #[test]
    fn mash_textures_works() {
        // arrange
        let mut images = Assets::<Image>::default();
        let red = images.add(create_image(
            (4, 4),
            TextureFormat::Rgba8UnormSrgb,
            [
                Color::RED, Color::RED,Color::RED, Color::RED,
                Color::RED, Color::RED, Color::RED, Color::RED,
                Color::RED, Color::RED, Color::RED, Color::RED,
                Color::RED, Color::RED, Color::RED, Color::RED,
            ],
        ));

        let green = images.add(create_image(
            (2, 2),
            TextureFormat::Rgba8UnormSrgb,
            [
                Color::GREEN, Color::GREEN,
                Color::GREEN, Color::GREEN
            ],
        ));

        let blue = images.add(create_image(
            (2, 2),
            TextureFormat::Rgba8UnormSrgb,
            [
                Color::BLUE, Color::BLUE,
                Color::BLUE, Color::BLUE
            ],
        ));

        // act
        let result = mash_textures(
            &mut images,
            [
                (Offset::new(0, 0, -1), red),
                (Offset::new(1, 1, 1), green),
                (Offset::new(2, 2, 0), blue),
            ]
        );

        // assert
        assert!(result.is_ok());

        let expected = create_image(
            (4, 4),
            TextureFormat::Rgba8UnormSrgb,
            [
                Color::RED, Color::RED,Color::RED, Color::RED,
                Color::RED, Color::GREEN, Color::GREEN, Color::RED,
                Color::RED, Color::GREEN, Color::GREEN, Color::BLUE,
                Color::RED, Color::RED, Color::BLUE, Color::BLUE,
            ],
        );
        let created_image = images.get(result.unwrap());

        assert_eq!(expected.data, created_image.unwrap().data);
    }
}