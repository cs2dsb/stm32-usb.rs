use packing::Packed;
use crate::scsi::{
    packing::ParsePackedStruct,
    commands::Control,
};

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct ModeSelectXCommand {
    // TBD
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, Packed)]
#[packed(big_endian, lsb0)]
pub struct ModeSelect6Command {
    #[pkd(4, 4, 0, 0)]
    pub page_format: bool,

    #[pkd(0, 0, 0, 0)]
    pub save_pages: bool,

    #[pkd(7, 0, 3, 3)]
    pub parameter_list_length: u8,

    #[pkd(7, 0, 4, 4)]
    pub control: Control,
}
impl ParsePackedStruct for ModeSelect6Command {}
impl From<ModeSelect6Command> for ModeSelectXCommand {
    fn from(_m: ModeSelect6Command) -> Self {
        Self { }
    }
}


#[derive(Clone, Copy, Eq, PartialEq, Debug, Packed)]
#[packed(big_endian, lsb0)]
pub struct ModeSelect10Command {
    #[pkd(4, 4, 0, 0)]
    pub page_format: bool,

    #[pkd(0, 0, 0, 0)]
    pub save_pages: bool,

    #[pkd(7, 0, 6, 7)]
    pub parameter_list_length: u16,

    #[pkd(7, 0, 8, 8)]
    pub control: Control,
}
impl ParsePackedStruct for ModeSelect10Command {}
impl From<ModeSelect10Command> for ModeSelectXCommand {
    fn from(_m: ModeSelect10Command) -> Self {
        Self { }
    }
}