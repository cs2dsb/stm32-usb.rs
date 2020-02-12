use packed_struct_codegen::PackedStruct;
use packed_struct::PackedStruct;
use crate::scsi::{
    packing::{ ResizeSmaller, ParsePackedStruct },
    commands::Control,
};

#[derive(Clone, Copy, Eq, PartialEq, Debug, PackedStruct)]
#[packed_struct(endian="msb", bit_numbering="msb0")]
pub struct StartStopUnitCommand {
    #[packed_field(bits="7..8")]
    pub immediate: bool,
    #[packed_field(bits="20..24")]
    pub power_condition_modifier: u8,
    #[packed_field(bits="24..28")]
    pub power_condition: u8,
    #[packed_field(bits="29..30")]
    pub no_flush: bool,
    #[packed_field(bits="30..31")]
    pub load_eject: bool,
    #[packed_field(bits="31..32")]
    pub start: bool,
    #[packed_field(bits="32..40")]
    pub control: Control,
}
impl<A: ResizeSmaller<[u8; StartStopUnitCommand::BYTES]>> ParsePackedStruct<A, [u8; StartStopUnitCommand::BYTES]> for StartStopUnitCommand {}