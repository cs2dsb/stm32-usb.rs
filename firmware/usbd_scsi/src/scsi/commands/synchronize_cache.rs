use packing::Packed;
use crate::scsi::{
    packing::ParsePackedStruct,
    commands::Control,
};

#[derive(Clone, Copy, Eq, PartialEq, Debug, Packed)]
#[packed(big_endian, lsb0)]
pub struct SynchronizeCache10Command {
    #[pkd(1, 1, 0, 0)]
    pub immediate: bool,
    
    #[pkd(7, 0, 1, 4)]
    pub lba: u32,
    
    #[pkd(4, 0, 5, 5)]
    pub group_number: u8,
    
    #[pkd(7, 0, 6, 7)]
    pub number_of_blocks: u16,
    
    #[pkd(7, 0, 8, 8)]
    pub control: Control,
}
impl ParsePackedStruct for SynchronizeCache10Command {}