use packed_struct_codegen::PackedStruct;
use packed_struct::PackedStruct;
use crate::scsi::{
    packing::{ ResizeSmaller, ParsePackedStruct },
    commands::Control,
};

#[derive(Clone, Copy, Eq, PartialEq, Debug, PackedStruct)]
#[packed_struct(endian="msb", bit_numbering="msb0")]
pub struct TestUnitReadyCommand {
    _reserved: u32,
    #[packed_field(element_size_bytes="1")]
    pub control: Control,
}

impl<A: ResizeSmaller<[u8; TestUnitReadyCommand::BYTES]>>  ParsePackedStruct<A, [u8; TestUnitReadyCommand::BYTES]> for TestUnitReadyCommand {}