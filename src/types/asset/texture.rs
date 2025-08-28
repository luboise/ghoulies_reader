use std::{
    fs::File,
    io::BufWriter,
    path::{Path, PathBuf},
};

use crate::types::{
    asset::{Asset, AssetDescriptor, AssetError, AssetParseError, BufferViewList},
    d3d::{D3DFormat, LinearColour},
    game::AssetType,
};

#[derive(Debug, Clone)]
pub struct TextureDescriptor {
    format: D3DFormat,
    header_size: u32, // 28
    width: u16,
    height: u16,
    flags: u32, // 0x00000001
    unknown_3a: u32,
    tile_count: u32,
    unknown_3c: u32,
    texture_offset: u32,
    data_size: u32,
}

#[derive(Debug)]
pub struct Texture {
    name: String,
    descriptor: TextureDescriptor,
    views: BufferViewList,
}

impl AssetDescriptor for TextureDescriptor {
    fn from_bytes(data: &[u8]) -> Result<Self, AssetError> {
        if data.len() < size_of::<TextureDescriptor>() {
            return Err(AssetError::AssetParseError(AssetParseError::InputTooSmall));
        }

        let format: D3DFormat = D3DFormat::Linear(LinearColour::A8B8G8R8);
        let header_size = u32::from_le_bytes(data[4..8].try_into().unwrap());
        let width = u16::from_le_bytes(data[8..10].try_into().unwrap());
        let height = u16::from_le_bytes(data[10..12].try_into().unwrap());
        let flags = u32::from_le_bytes(data[12..16].try_into().unwrap());
        let unknown_3a = u32::from_le_bytes(data[16..20].try_into().unwrap());
        let flags = u32::from_le_bytes(data[20..24].try_into().unwrap());
        let tile_count = u32::from_le_bytes(data[24..28].try_into().unwrap());
        let unknown_3c = u32::from_le_bytes(data[28..32].try_into().unwrap());
        let texture_offset = u32::from_le_bytes(data[32..36].try_into().unwrap());
        let data_size = u32::from_le_bytes(data[36..40].try_into().unwrap());

        Ok(TextureDescriptor {
            format,
            header_size,
            width,
            height,
            flags,
            unknown_3a,
            tile_count,
            unknown_3c,
            texture_offset,
            data_size,
        })
    }
}

impl Asset for Texture {
    type Descriptor = TextureDescriptor;

    fn new(
        name: &str,
        descriptor: &Self::Descriptor,
        views: &BufferViewList,
    ) -> Result<Self, AssetParseError> {
        Ok(Texture {
            name: name.to_string(),
            descriptor: descriptor.clone(),
            views: views.clone(),
        })
    }

    fn descriptor(&self) -> &Self::Descriptor {
        &self.descriptor
    }

    fn asset_type() -> AssetType {
        AssetType::ResTexture
    }

    fn buffer_views(&self) -> &BufferViewList {
        &self.views
    }

    fn name(&self) -> &str {
        &self.name
    }
}

impl Texture {
    pub fn dump(&self, path: &Path) -> Result<(), std::io::Error> {
        let mut p: PathBuf = path.to_path_buf();

        if p.is_dir() {
            p = p.join(format!("{}.png", self.name()));
        }

        let file = File::create(p).unwrap();
        let w = &mut BufWriter::new(file);

        let mut encoder = png::Encoder::new(
            w,
            self.descriptor.width as u32,
            self.descriptor.height as u32,
        ); // Width is 2 pixels and height is 1.

        let use_rgba = true;

        encoder.set_color(match use_rgba {
            true => png::ColorType::Rgba,
            false => png::ColorType::Rgb,
        });
        encoder.set_depth(png::BitDepth::Eight);

        // encoder.set_source_gamma(png::ScaledFloat::new(1.0 / 2.2));
        /*
        let chroma = png::SourceChromaticities::new(
            (0.3127, 0.3290), // red
            (0.6400, 0.3300), // green
            (0.3000, 0.6000), // blue
            (0.1500, 0.0600), // white
        );
        encoder.set_source_chromaticities(chroma);
        */

        let mut writer = encoder.write_header().unwrap();

        writer.write_image_data(&self.views.views[0].data).unwrap();
        writer.finish().expect("Unable to close writer.");

        Ok(())
    }
}
