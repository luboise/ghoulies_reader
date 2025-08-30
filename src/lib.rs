pub(crate) mod d3d;

pub(crate) mod images;

pub mod asset;

use byteorder::{LittleEndian, ReadBytesExt};

use std::{
    error::Error,
    io::{Cursor, Read, Seek, SeekFrom},
};

use crate::{
    asset::{
        Asset, AssetDescription, AssetDescriptor, AssetError, AssetName, AssetParseError,
        DataViewList, RawAsset,
    },
    game::AssetType,
};

pub mod game;

#[derive(Debug, Copy, Clone, Default)]
pub struct DataView {
    offset: u32,
    size: u32,
}

impl DataView {
    pub fn from_cursor<T>(cur: &mut Cursor<T>) -> Result<DataView, std::io::Error>
    where
        Cursor<T>: std::io::Read,
    {
        let offset = cur.read_u32::<LittleEndian>()?;
        let size = cur.read_u32::<LittleEndian>()?;

        Ok(DataView { offset, size })
    }
}

macro_rules! read {
    ($file:expr, u8) => {
        $file.read_u8()?
    };
    ($file:expr, u16) => {
        $file.read_u16::<LittleEndian>()?
    };
    ($file:expr, u32) => {
        $file.read_u32::<LittleEndian>()?
    };
    ($file:expr, u64) => {
        $file.read_u64::<LittleEndian>()?
    };
    ($file:expr, i32) => {
        $file.read_i32::<LittleEndian>()?
    };
}

#[derive(Debug)]
pub enum BNLError {
    /// The ZLIB portion of the BNL file could not be decompressed successfully.
    DecompressionFailure,
    /// An error occurred when parsing the [`AssetDescription`] data of the BNL file.
    DataReadError(String),
}

impl From<std::io::Error> for BNLError {
    fn from(value: std::io::Error) -> Self {
        BNLError::DataReadError(format!("File error: {}", value))
    }
}

impl From<miniz_oxide::inflate::DecompressError> for BNLError {
    fn from(_: miniz_oxide::inflate::DecompressError) -> Self {
        BNLError::DecompressionFailure
    }
}

#[derive(Debug, Default)]
struct BNLHeader {
    file_count: u16,
    flags: u8,
    unknown_2: [u8; 5],

    asset_desc_loc: DataView,
    buffer_views_loc: DataView,
    buffer_loc: DataView,
    descriptor_loc: DataView,
}

#[derive(Debug, Default)]
pub struct BNLFile {
    header: BNLHeader,

    asset_desc_bytes: Vec<u8>,
    buffer_views_bytes: Vec<u8>,
    buffer_bytes: Vec<u8>,
    descriptor_bytes: Vec<u8>,

    asset_descriptions: Vec<AssetDescription>,
}

impl BNLFile {
    /**
    Parses a BNL file in memory, loading embedded [`AssetDescription`] data.

    # Errors
    - [`BNLError::DecompressionFailure`] when the zlib compression section of the file could not be parsed
    - [`BNLError::DataReadError`] when any other part of the file could not be parsed

    # Examples
    ```
    use bnl::BNLFile;
    use std::path::PathBuf;

    let path = PathBuf::new("./my_bnl.bnl");
    let bytes = fs::read(&path).expect("Unable to read BNL.");

    let bnl = BNLFile::from_bytes(&bytes).expect("Unable to parse BNL.");
    ```
    */
    pub fn from_bytes(bnl_bytes: &[u8]) -> Result<BNLFile, BNLError> {
        let mut bytes = bnl_bytes[..40].to_vec();

        let mut cur = Cursor::new(bnl_bytes);

        let mut header = BNLHeader {
            file_count: read!(cur, u16),
            flags: read!(cur, u8),
            ..Default::default()
        };

        let decompressed_bytes = miniz_oxide::inflate::decompress_to_vec_zlib(&bytes[40..])?;
        bytes.extend_from_slice(&decompressed_bytes);

        cur.read_exact(&mut header.unknown_2)?;

        header.asset_desc_loc = DataView::from_cursor(&mut cur)?;
        header.buffer_views_loc = DataView::from_cursor(&mut cur)?;
        header.buffer_loc = DataView::from_cursor(&mut cur)?;
        header.descriptor_loc = DataView::from_cursor(&mut cur)?;

        let mut new_bnl = BNLFile {
            header,
            ..Default::default()
        };

        assert_eq!(size_of::<AssetDescription>(), 160);

        let num_descriptions =
            new_bnl.header.asset_desc_loc.size as usize / size_of::<AssetDescription>();

        for _ in 0..num_descriptions {
            let mut asset_name: AssetName = [0x00; 128];

            cur.read_exact(&mut asset_name)?;

            // TODO: Rework this into an actual constructor
            let asset_desc = AssetDescription {
                name: asset_name,
                asset_type: AssetType::try_from(read!(cur, u32)).map_err(|_| {
                    BNLError::DataReadError("Unable to parse asset type from BNL.".to_string())
                })?,
                unk_1: read!(cur, u32),
                unk_2: read!(cur, u32),
                chunk_count: read!(cur, u32),
                descriptor_ptr: read!(cur, u32),
                descriptor_size: read!(cur, u32),
                dataview_list_ptr: read!(cur, u32),
                resource_size: read!(cur, u32),
            };

            // TODO: Resize this then push into it
            new_bnl.asset_descriptions.push(asset_desc);
        }

        let loc = &new_bnl.header.buffer_views_loc;
        cur.seek(SeekFrom::Start(loc.offset.into()))?;
        new_bnl.buffer_views_bytes.resize(loc.size as usize, 0);
        cur.read_exact(&mut new_bnl.buffer_views_bytes)?;

        let loc = &new_bnl.header.buffer_loc;
        cur.seek(SeekFrom::Start(loc.offset.into()))?;
        new_bnl.buffer_bytes.resize(loc.size as usize, 0);
        cur.read_exact(&mut new_bnl.buffer_bytes)?;

        let loc = &new_bnl.header.descriptor_loc;
        cur.seek(SeekFrom::Start(loc.offset.into()))?;
        new_bnl.descriptor_bytes.resize(loc.size as usize, 0);
        cur.read_exact(&mut new_bnl.descriptor_bytes)?;

        Ok(new_bnl)
    }

    /// Retrieves an asset by name and type, creating it from the bytes of the BNL file.
    ///
    /// # Errors
    /// - [`AssetError::NotFound`] when the given name can't be found
    /// - [`AssetError::TypeMismatch`] when the asset is found, but doesn't match the requested type
    /// - [`AssetError::ParseError`] when the asset is found, the type matches but an error occurs while parsing the asset
    ///
    /// # Examples
    /// ```
    /// use bnl::BNLFile;
    /// use bnl::asset::Texture;
    ///
    /// let bnl_file = BNLFile::from_bytes(...);
    /// let tex = bnl_file.get_asset::<Texture>("aid_texture_mytexture_a_b")
    ///                   .expect("Unable to get texture.");
    /// ```
    pub fn get_asset<A: Asset>(&self, name: &str) -> Result<A, AssetError> {
        for asset_desc in &self.asset_descriptions {
            if asset_desc.name() == name {
                if asset_desc.asset_type() != A::asset_type() {
                    return Err(AssetError::TypeMismatch);
                }

                let descriptor_ptr: usize = asset_desc.descriptor_ptr() as usize;
                let desc_slice = &self.descriptor_bytes[descriptor_ptr..];

                let descriptor: A::Descriptor = A::Descriptor::from_bytes(desc_slice)?;

                let slices = self
                    .get_dataview_list(asset_desc.dataview_list_ptr as usize)
                    .map_err(|_| {
                        AssetError::ParseError(AssetParseError::InvalidDataViews(
                            "Unable to get data view list from BNL data.".to_string(),
                        ))
                    })?
                    .slices(&self.buffer_bytes)
                    .map_err(|_| {
                        AssetError::ParseError(AssetParseError::InvalidDataViews(
                            "Unable to get data from data slices.".to_string(),
                        ))
                    })?;

                let asset = A::new(asset_desc.name(), &descriptor, &slices)?;

                return Ok(asset);
            }
        }

        Err(AssetError::NotFound)
    }

    /// Returns all assets of a given type from this [`BNLFile`].
    ///
    /// # Examples
    ///
    /// ```
    /// use bnl::BNLFile;
    /// use bnl::asset::Texture;
    ///
    /// let bnl_file = BNLFile::from_bytes(...);
    /// let textures = bnl_file.get_assets::<Texture>();
    ///
    /// // Dump all of the textures here
    /// ```
    pub fn get_assets<A: Asset>(&self) -> Vec<A> {
        let mut assets = Vec::new();

        for asset_desc in &self.asset_descriptions {
            if asset_desc.asset_type() != A::asset_type() {
                continue;
            }

            let descriptor_ptr: usize = asset_desc.descriptor_ptr() as usize;
            let desc_slice = &self.descriptor_bytes[descriptor_ptr..];

            let descriptor: A::Descriptor = match A::Descriptor::from_bytes(desc_slice) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!(
                        "Error getting asset descriptor for {}\nError: {}",
                        asset_desc.name(),
                        e
                    );
                    continue;
                }
            };

            let dvl = match self.get_dataview_list(asset_desc.dataview_list_ptr as usize) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!(
                        "Error getting DataViewList for asset {}. Skipping this asset.",
                        asset_desc.name()
                    );
                    continue;
                }
            };

            let slices = match dvl.slices(&self.buffer_bytes) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!(
                        "Error retrieving data view slices for asset {}. Skipping this asset.",
                        asset_desc.name()
                    );
                    continue;
                }
            };

            match A::new(asset_desc.name(), &descriptor, &slices) {
                Ok(a) => assets.push(a),
                Err(e) => eprintln!(
                    "Failed to load asset \"{}\"\n    Error: {}",
                    asset_desc.name(),
                    e
                ),
            };
        }

        assets
    }

    /// Retrieves a [`RawAsset`] by name.
    ///
    /// # Errors
    /// Returns an [`AssetError`] if the asset can not be parsed from the [`BNLFile`].
    ///
    /// # Examples
    /// ```
    /// use bnl::BNLFile;
    /// use bnl::asset::Texture;
    ///
    /// let bnl_file = BNLFile::from_bytes(...);
    /// let raw_asset = bnl_file.get_raw_asset().expect("Unable to extract.");
    ///
    /// // Dump the data from the RawAsset
    /// std::fs::write("./descriptor", &raw_asset.descriptor_bytes).expect("Unable to write
    /// descriptor.");
    /// raw_asset.data_slices.iter().enumerate().for_each(|(i, slice)| {
    ///     std::fs::write(format!("./resource{}", i), &slice).expect("Unable to write resource.");
    /// });
    /// ```
    pub fn get_raw_asset(&self, name: &str) -> Result<RawAsset, AssetError> {
        for asset_desc in &self.asset_descriptions {
            if asset_desc.name() == name {
                let desc_ptr: usize = asset_desc.descriptor_ptr() as usize;
                let desc_size: usize = asset_desc.descriptor_size as usize;

                let desc_bytes: Vec<u8> =
                    self.descriptor_bytes[desc_ptr..desc_ptr + desc_size].to_vec();

                /*
                    .map_err(|e| {
                        AssetError::AssetParseError(AssetParseError::InvalidDataViews(
                            "bruh".to_string(),
                        ))
                    })?;
                */

                let dvl = self
                    .get_dataview_list(asset_desc.dataview_list_ptr as usize)
                    .map_err(|_| {
                        AssetError::ParseError(AssetParseError::InvalidDataViews(
                            "Unable to get data view list from BNL data.".to_string(),
                        ))
                    })?;

                let slices = dvl.slices(&self.buffer_bytes).map_err(|_| {
                    AssetError::ParseError(AssetParseError::InvalidDataViews(
                        "Unable to get data from data slices.".to_string(),
                    ))
                })?;

                return Ok(RawAsset {
                    name: asset_desc.name().to_string(),
                    descriptor_bytes: desc_bytes,
                    data_slices: slices.iter().map(|s| s.to_vec()).collect(),
                });
            }
        }

        Err(AssetError::NotFound)
    }

    /// Retrieves all [`RawAsset`] entries.
    ///
    /// # Examples
    /// ```
    /// use bnl::BNLFile;
    /// use bnl::asset::Texture;
    ///
    /// let bnl_file = BNLFile::from_bytes(...);
    /// let raw_assets = bnl_file.get_raw_assets().expect("Unable to extract.");
    ///
    /// // Dump the data from the RawAsset
    ///
    /// for raw_asset in raw_assets {
    ///     std::fs::write("./descriptor", &raw_asset.descriptor_bytes)
    ///                         .expect("Unable to write descriptor.");
    ///
    ///     raw_asset.data_slices.iter().enumerate().for_each(|(i, slice)| {
    ///         std::fs::write(format!("./resource{}", i), &slice)
    ///                         .expect("Unable to write resource.");;
    ///     });
    /// }
    /// ```
    pub fn get_raw_assets(&self) -> Vec<RawAsset> {
        let mut assets = Vec::new();

        let clo = |asset_desc: &AssetDescription| -> Result<RawAsset, AssetError> {
            let desc_ptr: usize = asset_desc.descriptor_ptr() as usize;
            let desc_size: usize = asset_desc.descriptor_size as usize;

            let desc_bytes: Vec<u8> =
                self.descriptor_bytes[desc_ptr..desc_ptr + desc_size].to_vec();

            /*
                .map_err(|e| {
                    AssetError::AssetParseError(AssetParseError::InvalidDataViews(
                        "bruh".to_string(),
                    ))
                })?;
            */

            let dvl = self
                .get_dataview_list(asset_desc.dataview_list_ptr as usize)
                .map_err(|_| {
                    AssetError::ParseError(AssetParseError::InvalidDataViews(
                        "Unable to get data view list from BNL data.".to_string(),
                    ))
                })?;

            let slices = dvl.slices(&self.buffer_bytes).map_err(|_| {
                AssetError::ParseError(AssetParseError::InvalidDataViews(
                    "Unable to get data from data slices.".to_string(),
                ))
            })?;

            Ok(RawAsset {
                name: asset_desc.name().to_string(),
                descriptor_bytes: desc_bytes,
                data_slices: slices.iter().map(|s| s.to_vec()).collect(),
            })
        };

        for asset_desc in &self.asset_descriptions {
            match clo(asset_desc) {
                Ok(asset) => {
                    assets.push(asset);
                }
                Err(e) => {
                    eprintln!(
                        "Error retrieving RawAsset for {}.\nError: {}",
                        asset_desc.name(),
                        e
                    );
                }
            }
        }

        assets
    }

    /// Returns a reference to the asset descriptions of this [`BNLFile`].
    pub fn asset_descriptions(&self) -> &[AssetDescription] {
        &self.asset_descriptions
    }

    fn get_dataview_list(&self, offset: usize) -> Result<DataViewList, Box<dyn Error>> {
        Ok(DataViewList::from_bytes(
            &self.buffer_views_bytes[offset..],
        )?)
    }
}
