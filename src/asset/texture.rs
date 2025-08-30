use std::{
    fs::File,
    io::BufWriter,
    path::{Path, PathBuf},
};

use crate::{
    asset::{Asset, AssetDescriptor, AssetError, AssetParseError},
    d3d::{D3DFormat, LinearColour, PixelBits, StandardFormat, Swizzled},
    game::AssetType,
    images,
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

impl TextureDescriptor {
    pub fn format(&self) -> D3DFormat {
        self.format
    }

    pub fn required_size(&self) -> usize {
        (self.width as usize * self.height as usize * self.format.bits_per_pixel()).div_ceil(8)
    }
}

#[derive(Debug)]
pub struct Texture {
    name: String,
    descriptor: TextureDescriptor,
    data: Vec<u8>,
}

impl AssetDescriptor for TextureDescriptor {
    fn from_bytes(data: &[u8]) -> Result<Self, AssetError> {
        if data.len() < size_of::<TextureDescriptor>() {
            return Err(AssetError::ParseError(AssetParseError::InputTooSmall));
        }

        let format = match u32::from_le_bytes(data[0..4].try_into().unwrap()) {
            0x00000012 => D3DFormat::Swizzled(Swizzled::B8G8R8A8),
            0x0000003f => D3DFormat::Swizzled(Swizzled::A8B8G8R8),
            0x00000040 => D3DFormat::Linear(LinearColour::A8R8G8B8),
            0x0000000c => D3DFormat::Standard(StandardFormat::DXT1),
            0x0000000e => D3DFormat::Standard(StandardFormat::DXT2Or3),
            0x0000000f => D3DFormat::Standard(StandardFormat::DXT4Or5),
            unknown_format => {
                println!(
                    "Unimplemented format found {}. Assuming A8B8G8R8.",
                    unknown_format
                );
                D3DFormat::Linear(LinearColour::A8R8G8B8)
            }
        };
        let header_size = u32::from_le_bytes(data[4..8].try_into().unwrap());
        let width = u16::from_le_bytes(data[8..10].try_into().unwrap());
        let height = u16::from_le_bytes(data[10..12].try_into().unwrap());
        let flags = u32::from_le_bytes(data[12..16].try_into().unwrap());
        let unknown_3a = u32::from_le_bytes(data[16..20].try_into().unwrap());
        let tile_count = u32::from_le_bytes(data[20..24].try_into().unwrap());
        let unknown_3c = u32::from_le_bytes(data[24..28].try_into().unwrap());
        let texture_offset = u32::from_le_bytes(data[28..32].try_into().unwrap());
        let data_size = u32::from_le_bytes(data[32..36].try_into().unwrap());

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
        data_slices: &[&[u8]],
    ) -> Result<Self, AssetParseError> {
        if data_slices.is_empty() {
            return Err(AssetParseError::InvalidDataViews(
                "Unable to create a Texture using 0 data views".to_string(),
            ));
        } else if data_slices.len() > 1 {
            println!(
                "A texture was attempted to created from multiple views. Continuing with the first slice."
            );
        }

        let view = data_slices[0];

        let required_size = descriptor.required_size();

        if view.len() < required_size {
            return Err(AssetParseError::InvalidDataViews(format!(
                "Required {} bytes for texture ({} available in view)",
                required_size,
                view.len()
            )));
        } else if view.len() > required_size {
            /* TODO: Make this not print out, or implement log levels. This message can clog up the
             * terminal easily.
            println!(
                "The data view being used for texture {} is too large. There may be a mismatch between the texture's descriptor and the input data. Continuing with the required amount only.",
                name
            );
            */
        }

        Ok(Texture {
            name: name.to_string(),
            descriptor: descriptor.clone(),
            data: view[..required_size].to_owned(),
        })
    }

    fn descriptor(&self) -> &Self::Descriptor {
        &self.descriptor
    }

    fn asset_type() -> AssetType {
        AssetType::ResTexture
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

        let mut bytes: Vec<u8> = self.data.clone();

        let desired_format: D3DFormat = match self.descriptor.format {
            D3DFormat::Linear(LinearColour::R8G8B8A8)
            | D3DFormat::Swizzled(Swizzled::A8B8G8R8)
            | D3DFormat::Swizzled(Swizzled::A8R8G8B8) => D3DFormat::Linear(LinearColour::R8G8B8A8),
            _ => {
                /*
                eprintln!(
                    "Unexpected format found during dump: {:?}. Attempting to dump anyway.",
                    self.descriptor.format
                );
                */

                D3DFormat::Linear(LinearColour::R8G8B8A8)
            }
        };

        if desired_format != self.descriptor.format {
            bytes = images::transcode(
                self.descriptor.width.into(),
                self.descriptor.height.into(),
                self.descriptor.format,
                desired_format,
                bytes.as_ref(),
            )?;
        }

        let file = File::create(p).unwrap();
        let w = &mut BufWriter::new(file);

        let mut encoder = png::Encoder::new(
            w,
            self.descriptor.width as u32,
            self.descriptor.height as u32,
        ); // Width is 2 pixels and height is 1.

        // TODO: Set this per texture type
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

        writer.write_image_data(&bytes)?;
        writer.finish().expect("Unable to close writer");

        Ok(())
    }
}
