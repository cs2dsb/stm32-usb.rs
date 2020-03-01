use packing::Packed;
use crate::scsi::{
    packing::ParsePackedStruct,
    commands::Control,
};

#[derive(Clone, Copy, Eq, PartialEq, Debug, Packed)]
#[packed(big_endian, lsb0)]
pub struct ReportLunsCommand {
    #[pkd(7, 0, 1, 1)]
    pub select_report: u8,
    
    #[pkd(7, 0, 5, 8)]
    pub allocation_length: u32,
    
    #[pkd(7, 0, 10, 10)]
    pub control: Control,
}
impl ParsePackedStruct for ReportLunsCommand {}