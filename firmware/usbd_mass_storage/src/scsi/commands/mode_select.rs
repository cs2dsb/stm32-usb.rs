use packed_struct_codegen::PackedStruct;
use packed_struct::PackedStruct;
use crate::scsi::{
    packing::{ ResizeSmaller, ParsePackedStruct },
    commands::Control,
};

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct ModeSelectXCommand {
    // TBD
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, PackedStruct)]
#[packed_struct(endian="msb", bit_numbering="msb0")]
pub struct ModeSelect6Command {
    #[packed_field(bits="3..4")]
    pub page_format: bool,
    #[packed_field(bits="6..7")]
    pub revert_to_defaults: bool,
    #[packed_field(bits="7..8")]
    pub save_pages: bool,
    #[packed_field(bits="24..32")]
    pub parameter_list_length: u8,
    #[packed_field(bits="32..40")]
    pub control: Control,
}
impl<A: ResizeSmaller<[u8; ModeSelect6Command::BYTES]>> ParsePackedStruct<A, [u8; ModeSelect6Command::BYTES]> for ModeSelect6Command {}
impl From<ModeSelect6Command> for ModeSelectXCommand {
    fn from(_m: ModeSelect6Command) -> Self {
        Self { }
    }
}


#[derive(Clone, Copy, Eq, PartialEq, Debug, PackedStruct)]
#[packed_struct(endian="msb", bit_numbering="msb0")]
pub struct ModeSelect10Command {
    #[packed_field(bits="3..4")]
    pub page_format: bool,
    #[packed_field(bits="7..8")]
    pub save_pages: bool,
    #[packed_field(bits="48..64")]
    pub parameter_list_length: u16,
    #[packed_field(bits="64..72")]
    pub control: Control,
}
impl<A: ResizeSmaller<[u8; ModeSelect10Command::BYTES]>> ParsePackedStruct<A, [u8; ModeSelect10Command::BYTES]> for ModeSelect10Command {}
impl From<ModeSelect10Command> for ModeSelectXCommand {
    fn from(_m: ModeSelect10Command) -> Self {
        Self { }
    }
}