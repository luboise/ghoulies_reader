pub(crate) mod d3d;

pub mod asset;

pub mod bnl;
pub use bnl::*;

use std::io::Cursor;

use byteorder::{LittleEndian, ReadBytesExt};

pub mod game;

pub type AssetName = [u8; 128];

#[derive(Debug, Copy, Clone, Default)]
pub struct DataView {
    offset: u32,
    size: u32,
}

impl DataView {
    pub fn from_cursor(cur: &mut Cursor<Vec<u8>>) -> Result<DataView, std::io::Error> {
        let offset = cur.read_u32::<LittleEndian>()?;
        let size = cur.read_u32::<LittleEndian>()?;

        Ok(DataView { offset, size })
    }
}
