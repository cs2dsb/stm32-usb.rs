use packing::Packed;
use crate::scsi::{
    packing::ParsePackedStruct,
    commands::Control,
};

#[derive(Clone, Copy, Eq, PartialEq, Debug, Packed)]
#[packed(big_endian, lsb0)]
pub struct StartStopUnitCommand {
    #[pkd(0, 0, 0, 0)]
    pub immediate: bool,
    
    #[pkd(3, 0, 2, 2)]
    pub power_condition_modifier: u8,
    
    #[pkd(7, 4, 3, 3)]
    pub power_condition: u8,
    
    #[pkd(2, 2, 3, 3)]
    pub no_flush: bool,
    
    #[pkd(1, 1, 3, 3)]
    pub load_eject: bool,
    
    #[pkd(0, 0, 3, 3)]
    pub start: bool,
    
    #[pkd(7, 0, 4, 4)]
    pub control: Control,
}
impl ParsePackedStruct for StartStopUnitCommand {}