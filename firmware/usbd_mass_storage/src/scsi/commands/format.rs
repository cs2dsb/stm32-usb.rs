use packed_struct_codegen::PackedStruct;
use packed_struct::PackedStruct;
use crate::scsi::{
    packing::{ ResizeSmaller, ParsePackedStruct },
    commands::Control,
};

#[derive(Clone, Copy, Eq, PartialEq, Debug, PackedStruct)]
#[packed_struct(endian="msb", bit_numbering="msb0")]
pub struct FormatCommand {
    #[packed_field(bits="0..2")]
    pub format_protection_information: u8,
    #[packed_field(bits="2..3")]
    pub long_list: bool,
    #[packed_field(bits="3..4")]
    pub format_data: bool,
    #[packed_field(bits="4..5")]
    pub complete_list: bool,
    #[packed_field(bits="5..8")]
    pub defect_list_format: u8,
    #[packed_field(bits="8..16")]
    pub vendor_specific: u8,
    #[packed_field(bits="30..32")]
    pub fast_format: u8,
    #[packed_field(bits="32..40")]
    pub control: Control,
}
impl<A: ResizeSmaller<[u8; FormatCommand::BYTES]>> ParsePackedStruct<A, [u8; FormatCommand::BYTES]> for FormatCommand {}