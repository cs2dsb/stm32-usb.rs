use packed_struct_codegen::PackedStruct;
use packed_struct::PackedStruct;
use crate::scsi::{
    packing::{ ResizeSmaller, ParsePackedStruct },
    commands::Control,
};

#[derive(Clone, Copy, Eq, PartialEq, Debug, PackedStruct)]
#[packed_struct(endian="msb", bit_numbering="msb0")]
pub struct SendDiagnosticCommand {
    #[packed_field(bits="0..3")]
    pub self_test_code: u8,
    #[packed_field(bits="3..4")]
    pub page_format: bool,
    #[packed_field(bits="5..6")]
    pub self_test: bool,
    #[packed_field(bits="6..7")]
    pub device_offline: bool,
    #[packed_field(bits="7..8")]
    pub unit_offline: bool,
    #[packed_field(bits="16..32")]
    pub parameter_list_length: u16,
    #[packed_field(bits="32..40")]
    pub control: Control,
}
impl<A: ResizeSmaller<[u8; SendDiagnosticCommand::BYTES]>> ParsePackedStruct<A, [u8; SendDiagnosticCommand::BYTES]> for SendDiagnosticCommand {}