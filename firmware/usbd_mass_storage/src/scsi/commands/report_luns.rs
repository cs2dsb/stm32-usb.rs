use packed_struct_codegen::PackedStruct;
use packed_struct::PackedStruct;
use crate::scsi::{
    packing::{ ResizeSmaller, ParsePackedStruct },
    commands::Control,
};

#[derive(Clone, Copy, Eq, PartialEq, Debug, PackedStruct)]
#[packed_struct(endian="msb", bit_numbering="msb0")]
pub struct ReportLunsCommand {
    #[packed_field(bits="8..16")]
    pub select_report: u8,
    #[packed_field(bits="40..72")]
    pub allocation_length: u32,
    #[packed_field(bits="80..88")]
    pub control: Control,
}
impl<A: ResizeSmaller<[u8; ReportLunsCommand::BYTES]>> ParsePackedStruct<A, [u8; ReportLunsCommand::BYTES]> for ReportLunsCommand {}