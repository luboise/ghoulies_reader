use crate::types::{ChunkLocator, game::AssetType};

pub mod texture;

#[derive(Debug)]
pub struct BufferViewList {
    size: u32,
    num_views: u32,
    views: Vec<BufferView>,
}

impl BufferViewList {
    pub fn from_bytes(bytes: &[u8]) -> Result<BufferViewList, Box<std::io::Error>> {
        if bytes.len() < 8 {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Data not long enough",
            )));
        };

        let mut size: u32 = 0;
        let mut num_views: u32 = 0;

        if let Ok(b) = bytes[0..8].try_into() {
            size = u32::from_le_bytes(b);
            num_views = u32::from_le_bytes(b[4..].try_into().unwrap());
        }

        if bytes.len() < num_views as usize * size_of::<ChunkLocator>() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Input is not large enough.",
            )
            .into());
        }

        let mut views = Vec::with_capacity(num_views as usize);

        bytes[8..]
            .chunks(size_of::<ChunkLocator>())
            .for_each(|chunk| {
                let view_offset = u32::from_le_bytes(chunk[0..4].try_into().unwrap());
                let view_size = u32::from_le_bytes(chunk[4..8].try_into().unwrap());

                views.push(BufferView {
                    offset: view_offset,
                    size: view_size,
                    data: Default::default(),
                });
            });

        Ok(BufferViewList {
            size,
            num_views,
            views,
        })
    }
}

#[derive(Debug)]
pub struct BufferView {
    offset: u32,
    size: u32,
    data: Vec<u8>,
}

#[derive(Debug)]
pub enum AssetParseError {
    ParserNotImplemented,
    ErrorParsingDescriptor,
    InputTooSmall,
}

#[derive(Debug)]
pub enum AssetError {
    AssetParseError(AssetParseError),
    AssetTypeMismatch,
    AssetNameNotFound,
}

impl From<AssetParseError> for AssetError {
    fn from(err: AssetParseError) -> Self {
        AssetError::AssetParseError(err)
    }
}

pub trait AssetDescriptor: Sized + Clone {
    fn from_bytes(data: &[u8]) -> Result<Self, AssetError>;
}

pub trait Asset: Sized {
    type Descriptor: AssetDescriptor;

    fn descriptor(&self) -> &Self::Descriptor;
    fn new(descriptor: &Self::Descriptor, views: &BufferViewList) -> Result<Self, AssetParseError>;

    fn asset_type() -> AssetType;

    fn buffer_views(&self) -> &Vec<BufferView>;
}
