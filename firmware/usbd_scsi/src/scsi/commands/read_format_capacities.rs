use packing::Packed;
use crate::scsi::{
    packing::ParsePackedStruct,
    commands::Control,
};

#[derive(Clone, Copy, Eq, PartialEq, Debug, Packed)]
#[packed(big_endian, lsb0)]
pub struct ReadFormatCapacitiesCommand {
    #[pkd(7, 5, 0, 0)]
    pub logical_unit_number: u8,
    
    #[pkd(7, 0, 6, 7)]
    pub allocation_length: u16,
    
    #[pkd(7, 0, 10, 10)]
    pub control: Control,
}
impl ParsePackedStruct for ReadFormatCapacitiesCommand {}