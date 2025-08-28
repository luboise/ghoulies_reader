pub(crate) mod d3d;

pub mod asset;

pub mod bnl;
pub use bnl::*;

use std::io::Cursor;

use byteorder::{LittleEndian, ReadBytesExt};

pub mod game;

pub type AssetName = [u8; 128];

#[derive(Debug, Copy, Clone, Default)]
struct ChunkLocator {
    offset: u32,
    size: u32,
}

impl ChunkLocator {
    pub fn from_cursor(cur: &mut Cursor<Vec<u8>>) -> Result<ChunkLocator, std::io::Error> {
        let offset = cur.read_u32::<LittleEndian>()?;
        let size = cur.read_u32::<LittleEndian>()?;

        Ok(ChunkLocator { offset, size })
    }
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
