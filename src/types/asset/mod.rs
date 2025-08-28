use std::fmt;

use crate::types::{ChunkLocator, game::AssetType};

pub mod texture;

#[derive(Debug, Clone)]
pub struct BufferViewList {
    size: u32,
    num_views: u32,
    views: Vec<BufferView>,
}

impl BufferViewList {
    pub fn from_bytes(
        view_bytes: &[u8],
        resource_bytes: &[u8],
    ) -> Result<BufferViewList, Box<std::io::Error>> {
        if view_bytes.len() < 8 {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Data not long enough",
            )));
        };

        let b = view_bytes[0..4]
            .try_into()
            .expect("slice with incorrect length");
        let size = u32::from_le_bytes(b);

        let b = view_bytes[4..8]
            .try_into()
            .expect("slice with incorrect length");
        let num_views = u32::from_le_bytes(b);

        if num_views == 0 || size != num_views * size_of::<ChunkLocator>() as u32 + 8 {
            return Err(Box::new(std::io::Error::other("Invalid size.")));
        }

        if view_bytes.len() < num_views as usize * size_of::<ChunkLocator>() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Input is not large enough.",
            )
            .into());
        }

        let mut views = Vec::with_capacity(num_views as usize);

        let mut chunks = view_bytes[8..].chunks(size_of::<ChunkLocator>());

        for _ in 0..num_views {
            let chunk = chunks.next().unwrap();

            let view_offset = u32::from_le_bytes(chunk[0..4].try_into().unwrap());
            let view_size = u32::from_le_bytes(chunk[4..8].try_into().unwrap());

            let data = resource_bytes
                [(view_offset as usize)..(view_offset as usize + view_size as usize)]
                .to_vec();

            views.push(BufferView {
                offset: view_offset,
                size: view_size,
                data,
            });
        }

        Ok(BufferViewList {
            size,
            num_views,
            views,
        })
    }
}

#[derive(Debug, Clone)]
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

impl fmt::Display for AssetError {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Write strictly the first element into the supplied output
        // stream: `f`. Returns `fmt::Result` which indicates whether the
        // operation succeeded or failed. Note that `write!` uses syntax which
        // is very similar to `println!`.
        write!(f, "Asset error")?;
        Ok(())
    }
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
    fn new(
        name: &str,
        descriptor: &Self::Descriptor,
        views: &BufferViewList,
    ) -> Result<Self, AssetParseError>;

    fn asset_type() -> AssetType;

    fn buffer_views(&self) -> &BufferViewList;

    fn name(&self) -> &str;
}
