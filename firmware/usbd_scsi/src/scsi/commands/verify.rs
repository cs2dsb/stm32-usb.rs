use packed_struct_codegen::PackedStruct;
use packed_struct::PackedStruct;
use crate::scsi::{
    packing::{ ResizeSmaller, ParsePackedStruct },
    commands::Control,
};

#[derive(Clone, Copy, Eq, PartialEq, Debug, PackedStruct)]
#[packed_struct(endian="msb", bit_numbering="msb0")]
pub struct Verify10Command {
    #[packed_field(bits="0..3")]
    pub vr_protect: u8,
    #[packed_field(bits="3..4")]
    pub dpo: bool,
    #[packed_field(bits="5..7")]
    pub byte_check: u8,
    #[packed_field(bits="8..40")]
    pub lba: u32,
    #[packed_field(bits="43..48")]
    pub group_number: u8,
    #[packed_field(bits="48..64")]
    pub verification_length: u16,
    #[packed_field(bits="64..72")]
    pub control: Control,
}
impl<A: ResizeSmaller<[u8; Verify10Command::BYTES]>> ParsePackedStruct<A, [u8; Verify10Command::BYTES]> for Verify10Command {}