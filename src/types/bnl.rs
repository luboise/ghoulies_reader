use byteorder::{LittleEndian, ReadBytesExt};
use std::{
    error::Error,
    io::{Cursor, Read, Seek, SeekFrom},
};

use crate::types::{
    AssetName, DataView,
    asset::{Asset, AssetDescriptor, AssetError, AssetParseError, DataViewList, RawAsset},
    game::AssetType,
};

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

pub struct AssetDescription {
    name: AssetName,
    asset_type: AssetType,

    unk_1: u32,
    unk_2: u32,
    chunk_count: u32,

    descriptor_ptr: u32,
    descriptor_size: u32,
    dataview_list_ptr: u32,
    resource_size: u32, // The total size needed for this asset, including its descriptor list
}

impl AssetDescription {
    fn has_raw_data(&self) -> bool {
        self.resource_size > 0
    }

    fn name(&self) -> &str {
        std::str::from_utf8(&self.name)
            .unwrap_or("")
            .split('\0')
            .next()
            .unwrap_or("")
    }

    fn asset_type(&self) -> AssetType {
        self.asset_type
    }

    fn unk_1(&self) -> u32 {
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
    pub fn get_dataview_list(&self, offset: usize) -> Result<DataViewList, Box<dyn Error>> {
        Ok(DataViewList::from_bytes(
            &self.buffer_views_bytes[offset..],
        )?)
    }

    pub fn asset_descriptions(&self) -> &[AssetDescription] {
        &self.asset_descriptions
    }

    pub fn get_asset<A: Asset>(&self, name: &str) -> Result<A, AssetError> {
        for asset_desc in &self.asset_descriptions {
            if asset_desc.name() == name {
                if asset_desc.asset_type() != A::asset_type() {
                    return Err(AssetError::AssetTypeMismatch);
                }

                let descriptor_ptr: usize = asset_desc.descriptor_ptr() as usize;
                let desc_slice = &self.descriptor_bytes[descriptor_ptr..];

                let descriptor: A::Descriptor = A::Descriptor::from_bytes(desc_slice)?;

                let slices = self
                    .get_dataview_list(asset_desc.dataview_list_ptr as usize)
                    .map_err(|_| {
                        AssetError::AssetParseError(AssetParseError::InvalidDataViews(
                            "Unable to get data view list from BNL data.".to_string(),
                        ))
                    })?
                    .slices(&self.buffer_bytes)
                    .map_err(|_| {
                        AssetError::AssetParseError(AssetParseError::InvalidDataViews(
                            "Unable to get data from data slices.".to_string(),
                        ))
                    })?;

                let asset = A::new(asset_desc.name(), &descriptor, &slices)?;

                return Ok(asset);
            }
        }

        Err(AssetError::AssetNameNotFound)
    }

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
                        AssetError::AssetParseError(AssetParseError::InvalidDataViews(
                            "Unable to get data view list from BNL data.".to_string(),
                        ))
                    })?;

                let slices = dvl.slices(&self.buffer_bytes).map_err(|_| {
                    AssetError::AssetParseError(AssetParseError::InvalidDataViews(
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

        Err(AssetError::AssetNameNotFound)
    }

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
                    AssetError::AssetParseError(AssetParseError::InvalidDataViews(
                        "Unable to get data view list from BNL data.".to_string(),
                    ))
                })?;

            let slices = dvl.slices(&self.buffer_bytes).map_err(|_| {
                AssetError::AssetParseError(AssetParseError::InvalidDataViews(
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

    pub fn from_cursor(cur: &mut Cursor<Vec<u8>>) -> Result<BNLFile, Box<dyn Error>> {
        let mut header = BNLHeader {
            file_count: read!(cur, u16),
            flags: read!(cur, u8),
            ..Default::default()
        };

        cur.read_exact(&mut header.unknown_2)?;

        header.asset_desc_loc = DataView::from_cursor(cur)?;
        header.buffer_views_loc = DataView::from_cursor(cur)?;
        header.buffer_loc = DataView::from_cursor(cur)?;
        header.descriptor_loc = DataView::from_cursor(cur)?;

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

            let asset_desc = AssetDescription {
                name: asset_name,
                asset_type: AssetType::try_from(read!(cur, u32))?,
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

    /*
    pub fn dump(&self, path: &PathBuf) -> Result<(), Box<dyn Error>> {
        println!("Dumping BNL file to {}", path.to_str().unwrap());
        if !path.exists() {
            println!("Creating output directory {}", path.to_str().unwrap());
            std::fs::create_dir(path)?;
        } else if path.is_file() {
            eprintln!(
                "Error: Unable to dump BNL file because the output directory, \"{}\" exists as a file.",
                path.to_str().unwrap()
            );
        }

        for asset in &self.asset_descriptions {
            let asset_folder = path.join(asset.name());

            if !asset_folder.exists() {
                std::fs::create_dir(&asset_folder)?;
            }

            for (i, resource) in asset.resources.iter().enumerate() {
                let file_path = asset_folder.join(format!("resource{}", i));
                if file_path.exists() && file_path.is_dir() {
                    eprintln!(
                        "Error: Path {} already exists but is a directory.",
                        file_path.to_str().unwrap()
                    );
                    panic!();
                }

                println!("Writing {}", file_path.to_str().unwrap());
                std::fs::write(file_path, &resource.data.as_slice())?;
            }
        }

        Ok(())
    }
    */
}
