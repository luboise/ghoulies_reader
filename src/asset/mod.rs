use std::{
    fmt::{self, Display},
    io,
};

use crate::{DataView, game::AssetType};

pub mod model;
pub mod texture;

#[derive(Debug, Clone)]
pub struct RawAsset {
    pub name: String,
    pub descriptor_bytes: Vec<u8>,
    pub data_slices: Vec<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub struct DataViewList {
    size: u32,
    num_views: u32,
    views: Vec<DataView>,
}

impl DataViewList {
    pub fn from_bytes(view_bytes: &[u8]) -> Result<DataViewList, Box<io::Error>> {
        if view_bytes.len() < 8 {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::InvalidData,
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

        if num_views == 0 || size != num_views * size_of::<DataView>() as u32 + 8 {
            return Err(Box::new(io::Error::other("Invalid size.")));
        }

        if view_bytes.len() < num_views as usize * size_of::<DataView>() {
            return Err(
                io::Error::new(io::ErrorKind::InvalidData, "Input is not large enough.").into(),
            );
        }

        let mut views = Vec::with_capacity(num_views as usize);

        let mut chunks = view_bytes[8..].chunks(size_of::<DataView>());

        for _ in 0..num_views {
            let chunk = chunks.next().unwrap();

            let view_offset = u32::from_le_bytes(chunk[0..4].try_into().unwrap());
            let view_size = u32::from_le_bytes(chunk[4..8].try_into().unwrap());

            views.push(DataView {
                offset: view_offset,
                size: view_size,
            });
        }

        Ok(DataViewList {
            size,
            num_views,
            views,
        })
    }

    pub fn slices<'a>(&self, data: &'a [u8]) -> Result<Vec<&'a [u8]>, io::Error> {
        if self.num_views as usize != self.views.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Invalid BufferViewList: num_views {} doesn't match the actual size of the views Vec {}",
                    self.num_views,
                    self.views.len()
                ),
            ));
        }

        Ok(self
            .views
            .iter()
            .map(|view| -> &[u8] {
                let start = view.offset as usize;
                let end = start + view.size as usize;
                &data[start..end]
            })
            .collect())
    }
}

#[derive(Debug)]
pub enum AssetParseError {
    /// The parser of a given type was not implemented, and the asset was not about to be parsed.
    // TODO: Remove this and just make it required by the trait
    ParserNotImplemented,
    /// An error occurred when parsing the [`Asset::Descriptor`] of the asset.
    ErrorParsingDescriptor,
    InputTooSmall,
    InvalidDataViews(String),
}

impl Display for AssetParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<std::io::Error> for AssetParseError {
    fn from(value: std::io::Error) -> Self {
        AssetParseError::InvalidDataViews("IO error occurred when parsing Asset.".to_string())
    }
}

#[derive(Debug)]
pub enum AssetError {
    /// The asset was found, but could not be parsed from the bytes of the [`crate::BNLFile`].
    ParseError(AssetParseError),
    /// The asset was found, but didn't match the expected [`AssetType`]
    TypeMismatch,
    /// The asset could not be found by name
    NotFound,
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
        AssetError::ParseError(err)
    }
}

/// Describes how a given asset is structured. Typically, an AssetDescriptor has information about how
/// to read an asset from its associated resources, as well as attributes of that asset. For
/// example, a [`texture::TextureDescriptor`] knows the width and height of its associated texture
/// resource.
pub trait AssetDescriptor: Sized + Clone {
    /// Creates a new AssetDescriptor from bytes.
    /// TODO: Finish the docs here
    fn from_bytes(data: &[u8]) -> Result<Self, AssetParseError>;
}

pub trait Asset: Sized {
    type Descriptor: AssetDescriptor;

    fn descriptor(&self) -> &Self::Descriptor;
    fn new(
        name: &str,
        descriptor: &Self::Descriptor,
        data_slices: &[&[u8]],
    ) -> Result<Self, AssetParseError>;

    fn asset_type() -> AssetType;

    fn name(&self) -> &str;
}

pub type AssetName = [u8; 128];

pub struct AssetDescription {
    pub(crate) name: AssetName,
    pub(crate) asset_type: AssetType,

    pub(crate) unk_1: u32,
    pub(crate) unk_2: u32,
    pub(crate) chunk_count: u32,

    pub(crate) descriptor_ptr: u32,
    pub(crate) descriptor_size: u32,
    pub(crate) dataview_list_ptr: u32,
    pub(crate) resource_size: u32, // The total size needed for this asset, including its descriptor list
}

impl AssetDescription {
    pub fn name(&self) -> &str {
        std::str::from_utf8(&self.name)
            .unwrap_or("")
            .split('\0')
            .next()
            .unwrap_or("")
    }

    // Getters
    pub fn has_raw_data(&self) -> bool {
        self.resource_size > 0
    }
    pub fn asset_type(&self) -> AssetType {
        self.asset_type
    }
    pub fn unk_1(&self) -> u32 {
        self.unk_1
    }
    pub fn bufferview_list_ptr(&self) -> u32 {
        self.dataview_list_ptr
    }
    pub fn resource_size(&self) -> u32 {
        self.resource_size
    }
    pub fn descriptor_ptr(&self) -> u32 {
        self.descriptor_ptr
    }
    pub fn descriptor_size(&self) -> u32 {
        self.descriptor_size
    }
}

impl std::fmt::Debug for AssetDescription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match str::from_utf8(&self.name) {
            Ok(s) => s.trim_matches('\x00'),
            Err(_) => return Err(std::fmt::Error {}),
        };
        f.debug_struct("HeaderEntry")
            .field("name", &name)
            .field("res_type", &self.asset_type)
            .field("unk_1", &self.unk_1)
            .field("unk_2", &self.unk_2)
            .field("chunk_count", &self.chunk_count)
            .field("descriptor_ptr", &self.descriptor_ptr)
            .field("descriptor_size", &self.descriptor_size)
            .field("bufferview_list_ptr", &self.dataview_list_ptr)
            .field("resource_size", &self.resource_size)
            .finish()
    }
}
