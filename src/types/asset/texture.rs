use crate::types::{
    asset::{Asset, AssetDescriptor, AssetError, AssetParseError, BufferView, BufferViewList},
    d3d::{D3DFormat, LinearColour, StandardFormat},
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
    descriptor: TextureDescriptor,
    data: Vec<u8>,
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

impl Texture {
    pub fn asset_type(&self) -> AssetType {
        AssetType::ResTexture
    }
}

impl Asset for Texture {
    type Descriptor = TextureDescriptor;

    fn new(descriptor: &Self::Descriptor, views: &BufferViewList) -> Result<Self, AssetParseError> {
        todo!("a");
    }

    fn descriptor(&self) -> &Self::Descriptor {
        &self.descriptor
    }

    fn asset_type() -> AssetType {
        AssetType::ResTexture
    }

    fn buffer_views(&self) -> &Vec<BufferView> {
        todo!()
    }
}
