use packing::Packed;
use crate::scsi::{
    packing::ParsePackedStruct,
    commands::Control,
};

#[derive(Clone, Copy, Eq, PartialEq, Debug, Packed)]
#[packed(big_endian, lsb0)]
pub struct PreventAllowMediumRemovalCommand {
    #[pkd(1, 0, 3, 3)]
    pub prevent: u8,

    #[pkd(7, 0, 4, 4)]
    pub control: Control,
}
impl ParsePackedStruct for PreventAllowMediumRemovalCommand {}