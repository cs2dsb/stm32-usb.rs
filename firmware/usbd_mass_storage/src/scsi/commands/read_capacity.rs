use packed_struct_codegen::PackedStruct;
use packed_struct::PackedStruct;
use crate::scsi::{
    packing::{ ResizeSmaller, ParsePackedStruct },
    commands::Control,
};

#[derive(Clone, Copy, Eq, PartialEq, Debug, PackedStruct)]
#[packed_struct(endian="msb", bit_numbering="msb0")]
pub struct ReadCapacity10Command {
    #[packed_field(bits="64..72")]
    pub control: Control,
}
impl<A: ResizeSmaller<[u8; ReadCapacity10Command::BYTES]>> ParsePackedStruct<A, [u8; ReadCapacity10Command::BYTES]> for ReadCapacity10Command {}