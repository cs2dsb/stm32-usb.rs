use packed_struct_codegen::PackedStruct;
use packed_struct::PackedStruct;
use crate::scsi::{
    packing::{ ResizeSmaller, ParsePackedStruct },
    commands::Control,
};

#[derive(Clone, Copy, Eq, PartialEq, Debug, PackedStruct)]
#[packed_struct(endian="msb", bit_numbering="msb0")]
pub struct SynchronizeCache10Command {
    #[packed_field(bits="6..7")]
    pub immediate: bool,
    #[packed_field(bits="8..40")]
    pub lba: u32,
    #[packed_field(bits="43..48")]
    pub group_number: u8,
    #[packed_field(bits="48..64")]
    pub number_of_blocks: u16,
    #[packed_field(bits="64..72")]
    pub control: Control,
}
impl<A: ResizeSmaller<[u8; SynchronizeCache10Command::BYTES]>> ParsePackedStruct<A, [u8; SynchronizeCache10Command::BYTES]> for SynchronizeCache10Command {}