use std::{
    fmt::{self, Display, Write},
    io,
};

use crate::types::{DataView, game::AssetType};

pub mod texture;

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
    ParserNotImplemented,
    ErrorParsingDescriptor,
    InputTooSmall,
    InvalidDataViews(String),
}

impl Display for AssetParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
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
        data_slices: &Vec<&[u8]>,
    ) -> Result<Self, AssetParseError>;

    fn asset_type() -> AssetType;

    fn name(&self) -> &str;
}
