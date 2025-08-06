use byteorder::{LittleEndian, ReadBytesExt};
use std::{
    error::Error,
    io::{Cursor, Read},
};

use crate::constants::ResourceType;

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

struct AssetHeader {
    asset_type: u32,
}

#[derive(Debug)]
struct ChunkLocator {
    offset: u32,
    size: u32,
}

struct HeaderEntry {
    name: AssetName,
    res_type: ResourceType,
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
        f.debug_struct("HeaderEntry").field("name", &name).finish()
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
    chunk1: Chunk1,
    // c1: ChunkLocator,
    // c2: ChunkLocator,
    // c3: ChunkLocator,
    // c4: ChunkLocator,
    /*
        def __str__(self):
            return "\n".join([f"File count: {self.file_count}", f"Flags: {self.flags:08X}", f"Unknowns: {self.unknown_2}", f"Headers:\n  {"\n  ".join([str(x) for x in self.headers])}", "\n"])

    */
}

#[derive(Debug)]
struct Chunk1 {
    locator: ChunkLocator,
}

// const u32 c1_count(m_header.chunk_1.size / sizeof(CHUNK_1_HEADER));

impl BNLFile {
    pub fn from_cursor(cur: &mut Cursor<Vec<u8>>) -> Result<BNLFile, Box<dyn Error>> {
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

        let mut header_entries = Vec::new();

        assert_eq!(size_of::<HeaderEntry>(), 160);

        for _ in 0..(locators[0].size as usize / size_of::<HeaderEntry>()) {
            let mut asset_name: AssetName = [0x00; 128];

            cur.read_exact(&mut asset_name)?;

            header_entries.push(HeaderEntry {
                name: asset_name,
                res_type: ResourceType::try_from(read!(cur, u32))?,
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
            });
        }

        for entry in header_entries {}

        Ok(BNLFile {
            header,
            chunk1: Chunk1 {
                locator: ChunkLocator { offset: 0, size: 0 },
            },
            // c1: todo!(),
            // c2: todo!(),
            // c3: todo!(),
            // c4: todo!(),
        })
    }
}
