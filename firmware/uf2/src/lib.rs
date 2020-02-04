#![no_std]
use core::ops::Deref;
use bitmask::bitmask;
use packed_struct_codegen::PackedStruct;
use failure::{ Fallible, Fail };

pub const DATA_LENGTH: usize = 476;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PackedStruct)]
#[packed_struct(endian="lsb")]
pub struct MagicStart {
    magic_0: u32,
    magic_1: u32,
}

impl Default for MagicStart {
    fn default() -> Self {
        const MAGIC_START0: u32 = 0x0A324655; // "UF2\n"
        const MAGIC_START1: u32 = 0x9E5D5157; // Randomly selected
        Self {
            magic_0: MAGIC_START0,
            magic_1: MAGIC_START1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PackedStruct)]
#[packed_struct(endian="lsb")]
pub struct MagicEnd {
    magic: u32,
}

impl Default for MagicEnd {
    fn default() -> Self {
        const MAGIC_END: u32 = 0x0AB16F30; // Randomly selected
        Self {
            magic: MAGIC_END,
        }
    }
}

bitmask! {
    #[derive(Debug, Default, PackedStruct)]
    #[packed_struct(endian="lsb")]
    pub mask Flags: u32 where 
    
    #[derive(Debug)]
    flags Flag {
        /// Block should be skipped when writing the device flash; it can be used to store "comments" in the file, typically embedded source code or debug info that does not fit on the device flash
        NotMainFlash = 0x00000001, 
        /// Block contains part of a file to be written to some kind of filesystem on the device
        FileContainer = 0x00001000,
        /// When set, the file_size_or_family_id holds a value identifying the board family (usually corresponds to an MCU)
        FamilyIdPresent = 0x00002000,
        /// When set, the last 24 bytes of data contain an Md5Checksum
        Md5ChecksumPresent = 0x00004000,
    }
}


#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Md5Checksum {
    address: u32,
    length: u32,
    checksum: [u8; 2],
}


#[derive(Clone, PackedStruct)]
#[packed_struct(endian="lsb")]
pub struct Data { 
    // Unfortunately packed struct requires a number here, a const won't work
    data: [u8; 476] 
}

impl Default for Data {
    fn default() -> Self {
        Self {
            data: [0; DATA_LENGTH],
        }
    }
}

impl Deref for Data {
    type Target=[u8; DATA_LENGTH];
    fn deref(&self) -> &[u8; DATA_LENGTH] {
        &self.data
    }
}

#[derive(Clone, Default, PackedStruct)]
#[packed_struct(endian="lsb")]
pub struct Block {
    #[packed_field(element_size_bytes="8")]
    magic_start: MagicStart,
    #[packed_field(element_size_bytes="4")]
    pub flags: Flags,
    pub target_address: u32,
    pub payload_size: u32,
    pub block_number: u32,
    pub number_of_blocks: u32,
    pub file_size_or_family_id: u32,
    #[packed_field(element_size_bytes="476")]
    pub data: Data,
    #[packed_field(element_size_bytes="4")]
    magic_end: MagicEnd,
}

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Data provided is too long for maximum UF2 block")]
    DataTooLong,
}

impl Block {
    pub fn new(target_address: u32, data: &[u8]) -> Fallible<Self> {
        if data.len() > DATA_LENGTH {
            Err(Error::DataTooLong)?
        }

        let payload_size = data.len() as u32;

        let mut new_block = Self {
            target_address,
            payload_size,
            .. Self::default()
        };

        new_block.data.data[..data.len()].copy_from_slice(data);

        Ok(new_block)
    }

    pub fn pack(&self) -> [u8; 512] {
        packed_struct::PackedStruct::pack(self)
    }
}