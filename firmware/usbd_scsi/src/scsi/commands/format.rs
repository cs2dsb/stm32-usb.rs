use packing::Packed;
use crate::scsi::{
    packing::ParsePackedStruct,
    commands::Control,
};

#[derive(Clone, Copy, Eq, PartialEq, Debug, Packed)]
#[packed(big_endian, lsb0)]
pub struct FormatCommand {
    #[pkd(7, 6, 0, 0)]
    pub format_protection_information: u8,
    
    #[pkd(5, 5, 0, 0)]
    pub long_list: bool,
    
    #[pkd(4, 4, 0, 0)]
    pub format_data: bool,
    
    #[pkd(3, 3, 0, 0)]
    pub complete_list: bool,
    
    #[pkd(2, 0, 0, 0)]
    pub defect_list_format: u8,
            
    #[pkd(7, 0, 4, 4)]
    pub control: Control,
}
impl ParsePackedStruct for FormatCommand {}