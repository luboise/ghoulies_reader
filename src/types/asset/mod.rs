use crate::types::game::AssetType;

pub mod texture;

#[derive(Debug)]
struct BufferView {
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
    fn from_descriptor(descriptor: &Self::Descriptor) -> Result<Self, AssetParseError>;

    fn asset_type() -> AssetType;

    fn buffer_views(&self) -> &Vec<BufferView>;
}
