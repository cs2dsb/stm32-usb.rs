use packed_struct_codegen::PackedStruct;
use packed_struct::PackedStruct;
use crate::scsi::{
    packing::{ ResizeSmaller, ParsePackedStruct },
    commands::Control,
};

#[derive(Clone, Copy, Eq, PartialEq, Debug, PackedStruct)]
#[packed_struct(endian="msb", bit_numbering="msb0")]
pub struct PreventAllowMediumRemovalCommand {
    #[packed_field(bits="30..32")]
    pub prevent: u8,
    #[packed_field(bits="32..40")]
    pub control: Control,
}
impl<A: ResizeSmaller<[u8; PreventAllowMediumRemovalCommand::BYTES]>> ParsePackedStruct<A, [u8; PreventAllowMediumRemovalCommand::BYTES]> for PreventAllowMediumRemovalCommand {}