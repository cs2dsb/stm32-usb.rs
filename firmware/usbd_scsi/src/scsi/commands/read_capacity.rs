use packing::Packed;
use crate::scsi::{
    packing::ParsePackedStruct,
    commands::Control,
};

#[derive(Clone, Copy, Eq, PartialEq, Debug, Packed)]
#[packed(big_endian, lsb0)]
pub struct ReadCapacity10Command {
    #[pkd(7, 0, 1, 4)]
    pub lba: u32,
    #[pkd(7, 0, 8, 8)]
    pub control: Control,
}
impl ParsePackedStruct for ReadCapacity10Command {}