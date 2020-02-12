use packed_struct_codegen::PackedStruct;
use packed_struct::PackedStruct;
use crate::scsi::{
    packing::{ ResizeSmaller, ParsePackedStruct },
    commands::Control,
};

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct WriteXCommand {
    pub lba: u32,
    pub transfer_length: u32,
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, PackedStruct)]
#[packed_struct(endian="msb", bit_numbering="msb0")]
pub struct Write6Command {
    #[packed_field(bits="3..24")]
    pub lba: u32,
    #[packed_field(bits="24..32")]
    pub transfer_length: u8,
    #[packed_field(bits="32..40")]
    pub control: Control,
}
impl<A: ResizeSmaller<[u8; Write6Command::BYTES]>>  ParsePackedStruct<A, [u8; Write6Command::BYTES]> for Write6Command {}
impl From<Write6Command> for WriteXCommand {
    fn from(w: Write6Command) -> Self {
        Self {
            lba: w.lba.into(),
            transfer_length: w.transfer_length.into(),
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, PackedStruct)]
#[packed_struct(endian="msb", bit_numbering="msb0")]
pub struct Write10Command {
    #[packed_field(bits="0..3")]
    pub wr_protect: u8,
    #[packed_field(bits="3..4")]
    pub dpo: bool,
    #[packed_field(bits="4..5")]
    pub fua: bool,
    #[packed_field(bits="8..40")]
    pub lba: u32,
    #[packed_field(bits="43..48")]
    pub group_number: u8,
    #[packed_field(bits="48..64")]
    pub transfer_length: u16,
    #[packed_field(bits="64..72")]
    pub control: Control,
}
impl<A: ResizeSmaller<[u8; Write10Command::BYTES]>>  ParsePackedStruct<A, [u8; Write10Command::BYTES]> for Write10Command {}
impl From<Write10Command> for WriteXCommand {
    fn from(w: Write10Command) -> Self {
        Self {
            lba: w.lba.into(),
            transfer_length: w.transfer_length.into(),
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, PackedStruct)]
#[packed_struct(endian="msb", bit_numbering="msb0")]
pub struct Write12Command {
    #[packed_field(bits="0..3")]
    pub wr_protect: u8,
    #[packed_field(bits="3..4")]
    pub dpo: bool,
    #[packed_field(bits="4..5")]
    pub fua: bool,
    #[packed_field(bits="8..40")]
    pub lba: u32,
    #[packed_field(bits="40..72")]
    pub transfer_length: u32,
    #[packed_field(bits="74..79")]
    pub group_number: u8,
    #[packed_field(bits="79..87")]
    pub control: Control,
}
impl<A: ResizeSmaller<[u8; Write12Command::BYTES]>>  ParsePackedStruct<A, [u8; Write12Command::BYTES]> for Write12Command {}
impl From<Write12Command> for WriteXCommand {
    fn from(w: Write12Command) -> Self {
        Self {
            lba: w.lba.into(),
            transfer_length: w.transfer_length.into(),
        }
    }
}