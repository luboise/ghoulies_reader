use crate::{
    asset::{AssetParseError, texture::TextureDescriptor},
    d3d::{D3DFormat, LinearColour, StandardFormat, Swizzled},
};

#[derive(Debug, Clone)]
pub struct SubTextureDescriptor {
    format: D3DFormat,
    header_size: u32, // 28
    width: u16,
    height: u16,
    flags: u32, // 0x00000001
    unknown_3a: u32,
    texture_offset: u32,
    texture_size: u32,
}

impl From<SubTextureDescriptor> for TextureDescriptor {
    fn from(value: SubTextureDescriptor) -> Self {
        TextureDescriptor::new(
            value.format,
            value.header_size,
            value.width,
            value.height,
            value.flags,
            value.unknown_3a,
            // TODO: Check that these are valid
            0,
            0,
            value.texture_offset,
            value.texture_size,
        )
    }
}

impl SubTextureDescriptor {
    pub fn from_bytes(data: &[u8]) -> Result<Self, AssetParseError> {
        if data.len() < size_of::<Self>() {
            return Err(AssetParseError::InputTooSmall);
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
        let texture_offset = u32::from_le_bytes(data[20..24].try_into().unwrap());
        let data_size = u32::from_le_bytes(data[28..32].try_into().unwrap());

        Ok(SubTextureDescriptor {
            format,
            header_size,
            width,
            height,
            flags,
            unknown_3a,
            texture_offset,
            texture_size: data_size,
        })
    }

    pub fn texture_offset(&self) -> u32 {
        self.texture_offset
    }

    pub fn texture_size(&self) -> u32 {
        self.texture_size
    }
}
