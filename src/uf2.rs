use std::io::Read;
use std::{io, mem};

pub const MAGIC_START0: u32 = 0x0A324655;
pub const MAGIC_START1: u32 = 0x9E5D5157;
pub const MAGIC_END: u32 = 0x0AB16F30;

const FAMILY_ID_PRESENT_FLAG: u32 = 0x00002000;

#[repr(C)]
pub struct UF2Block {
    magic_start0: u32,
    magic_start1: u32,
    flags: u32,
    target_addr: u32,
    pub payload_size: u32,
    pub block_no: u32,
    pub num_blocks: u32,
    file_size: u32, // or familyID
    data: [u8; 476],
    magic_end: u32,
}

type UF2BlockData = [u8; mem::size_of::<UF2Block>()];

impl UF2Block {
    fn from_bytes(data: UF2BlockData) -> Self {
        let mut block = unsafe { mem::transmute::<UF2BlockData, UF2Block>(data) };
        block.magic_start0 = u32::from_le(block.magic_start0);
        block.magic_start1 = u32::from_le(block.magic_start1);
        block.flags = u32::from_le(block.flags);
        block.target_addr = u32::from_le(block.target_addr);
        block.payload_size = u32::from_le(block.payload_size);
        block.block_no = u32::from_le(block.block_no);
        block.num_blocks = u32::from_le(block.num_blocks);
        block.file_size = u32::from_le(block.file_size);
        block.magic_end = u32::from_le(block.magic_end);
        block
    }

    pub fn family_id(&self) -> Option<u32> {
        if self.flags & FAMILY_ID_PRESENT_FLAG != 0 {
            Some(self.file_size)
        } else {
            None
        }
    }
}

pub struct U2FBlockIter<R> {
    read: R,
}

pub trait IntoU2FBlockIter<R> {
    fn u2f_block_iter(self) -> U2FBlockIter<R>;
}

impl<R> IntoU2FBlockIter<R> for R
where
    R: Read,
{
    fn u2f_block_iter(self) -> U2FBlockIter<R> {
        U2FBlockIter { read: self }
    }
}

impl<R> Iterator for U2FBlockIter<R>
where
    R: Read,
{
    type Item = Result<UF2Block, io::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut buffer: UF2BlockData = [0; mem::size_of::<UF2BlockData>()];
        // TODO: add error for incorrect number of bytes
        loop {
            match self.read.read(&mut buffer) {
                Ok(0) => return None,
                Ok(len) if len == mem::size_of::<UF2BlockData>() => {
                    let block = UF2Block::from_bytes(buffer);
                    if block.magic_start0 == MAGIC_START0
                        && block.magic_start1 == MAGIC_START1
                        && block.magic_end == MAGIC_END
                    {
                        return Some(Ok(block));
                    }
                }
                Ok(_) => unimplemented!(),
                Err(err) => return Some(Err(err)),
            }
        }
    }
}
