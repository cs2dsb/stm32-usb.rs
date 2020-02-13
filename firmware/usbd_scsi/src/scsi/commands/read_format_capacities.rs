use packed_struct_codegen::PackedStruct;
use packed_struct::PackedStruct;
use crate::scsi::{
    packing::{ ResizeSmaller, ParsePackedStruct },
    commands::Control,
};

#[derive(Clone, Copy, Eq, PartialEq, Debug, PackedStruct)]
#[packed_struct(endian="msb", bit_numbering="msb0")]
pub struct ReadFormatCapacitiesCommand {
    #[packed_field(bits="0..3")]
    pub logical_unit_number: u8,
    #[packed_field(bits="48..64")]
    pub allocation_length: u16,
    #[packed_field(bits="80..88")]
    pub control: Control,
}
impl<A: ResizeSmaller<[u8; ReadFormatCapacitiesCommand::BYTES]>> ParsePackedStruct<A, [u8; ReadFormatCapacitiesCommand::BYTES]> for ReadFormatCapacitiesCommand {}