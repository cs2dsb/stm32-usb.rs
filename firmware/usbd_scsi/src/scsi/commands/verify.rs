use packing::Packed;
use crate::scsi::{
    packing::ParsePackedStruct,
    commands::Control,
};

#[derive(Clone, Copy, Eq, PartialEq, Debug, Packed)]
#[packed(big_endian, lsb0)]
pub struct Verify10Command {
    #[pkd(7, 5, 0, 0)]
    pub vr_protect: u8,
    
    #[pkd(4, 4, 0, 0)]
    pub dpo: bool,
    
    #[pkd(1, 1, 0, 0)]
    pub byte_check: u8,
    
    #[pkd(7, 0, 1, 4)]
    pub lba: u32,
    
    #[pkd(4, 0, 5, 5)]
    pub group_number: u8,
    
    #[pkd(7, 0, 6, 7)]
    pub verification_length: u16,
    
    #[pkd(7, 0, 8, 8)]
    pub control: Control,
}
impl ParsePackedStruct for Verify10Command {}