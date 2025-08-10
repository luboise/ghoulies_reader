use byteorder::{LittleEndian, ReadBytesExt};
use std::{
    error::Error,
    io::{Cursor, Read, Seek, SeekFrom},
    path::PathBuf,
};

use crate::constants::AssetType;

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

pub type AssetName = [u8; 128];

#[derive(Debug, Copy, Clone)]
struct ChunkLocator {
    offset: u32,
    size: u32,
}

struct HeaderEntry {
    name: AssetName,
    res_type: AssetType,
    unk_1: u32,
    unk_2: u32,
    chunk_count: u32,
    file_loc: ChunkLocator,
    res_loc: ChunkLocator,
}

impl std::fmt::Debug for HeaderEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match str::from_utf8(&self.name) {
            Ok(s) => s.trim_matches('\x00'),
            Err(_) => return Err(std::fmt::Error {}),
        };
        f.debug_struct("HeaderEntry")
            .field("name", &name)
            .field("res_type", &self.res_type)
            .field("unk_1", &self.unk_1)
            .field("unk_2", &self.unk_2)
            .field("chunk_count", &self.chunk_count)
            .field("file_loc", &self.file_loc)
            .field("res_loc", &self.res_loc)
            .finish()
    }
}

#[derive(Debug)]
struct BNLHeader {
    file_count: u16,
    flags: u8,
    unknown_2: [u8; 5],
}

#[derive(Debug)]
pub struct BNLFile {
    header: BNLHeader,
    assets: Vec<Asset>,
}

struct AssetResource {
    offset: u32,
    size: u32,
    data: Vec<u8>,
}

impl std::fmt::Debug for AssetResource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AssetResource")
            .field("offset", &self.offset)
            .field("size", &self.size)
            .field("data", &format!("{} bytes", self.data.len()))
            .finish()
    }
}

#[derive(Debug)]
struct Asset {
    name: AssetName,
    res_type: AssetType,
    unk_1: u32,
    unk_2: u32,
    chunk_count: u32,
    file_loc: ChunkLocator,
    res_loc: ChunkLocator, // Where the asset was relative to the original header in the binary

    resources_size: u32,
    resource_count: u32,
    resources: Vec<AssetResource>,
}

impl Asset {
    fn name(&self) -> &str {
        std::str::from_utf8(&self.name)
            .unwrap_or("")
            .split('\0')
            .next()
            .unwrap_or("")
    }
}

// const u32 c1_count(m_header.chunk_1.size / sizeof(CHUNK_1_HEADER));

impl BNLFile {
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

        for asset in &self.assets {
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

    pub fn from_cursor(cur: &mut Cursor<Vec<u8>>) -> Result<BNLFile, Box<dyn Error>> {
        println!("Reading BNL file.");
        let mut header = BNLHeader {
            file_count: read!(cur, u16),
            flags: read!(cur, u8),
            unknown_2: [0, 0, 0, 0, 0],
        };
        cur.read_exact(&mut header.unknown_2)?;

        let locators: [ChunkLocator; 4] = [
            ChunkLocator {
                offset: read!(cur, u32),
                size: read!(cur, u32),
            },
            ChunkLocator {
                offset: read!(cur, u32),
                size: read!(cur, u32),
            },
            ChunkLocator {
                offset: read!(cur, u32),
                size: read!(cur, u32),
            },
            ChunkLocator {
                offset: read!(cur, u32),
                size: read!(cur, u32),
            },
        ];

        println!("Headers processed successfully");

        let mut assets = Vec::new();

        assert_eq!(size_of::<HeaderEntry>(), 160);

        println!("Beginning asset reading");

        for _ in 0..(locators[0].size as usize / size_of::<HeaderEntry>()) {
            let mut asset_name: AssetName = [0x00; 128];

            cur.read_exact(&mut asset_name)?;

            let mut asset = Asset {
                name: asset_name,
                res_type: AssetType::try_from(read!(cur, u32))?,
                unk_1: read!(cur, u32),
                unk_2: read!(cur, u32),
                chunk_count: read!(cur, u32),
                file_loc: ChunkLocator {
                    offset: read!(cur, u32),
                    size: read!(cur, u32),
                },
                res_loc: ChunkLocator {
                    offset: read!(cur, u32),
                    size: read!(cur, u32),
                },
                resources_size: 0,
                resource_count: 0,
                resources: Vec::new(),
            };

            print!("{} ", asset.name());

            let mut res_cur = cur.clone();
            res_cur.seek(SeekFrom::Start(
                (asset.res_loc.offset + locators[1].offset) as u64,
            ))?;

            asset.resources_size = read!(res_cur, u32);
            asset.resource_count = read!(res_cur, u32);

            for _ in 0..asset.resource_count {
                // Read the offset and size of the resource
                let mut resource = AssetResource {
                    offset: read!(res_cur, u32),
                    size: read!(res_cur, u32),
                    data: Vec::new(),
                };

                // println!("  Reading resource {}", i);

                let mut data_cur = res_cur.clone();
                data_cur.seek(SeekFrom::Start(
                    (locators[2].offset + resource.offset) as u64,
                ))?;

                resource.data.resize(resource.size as usize, 0);

                data_cur.read_exact(resource.data.as_mut_slice())?;

                asset.resources.push(resource);
            }

            assets.push(asset);
        }

        Ok(BNLFile { header, assets })
    }
}
