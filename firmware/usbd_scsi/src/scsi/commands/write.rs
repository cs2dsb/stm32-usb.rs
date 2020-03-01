use packing::Packed;
use crate::scsi::{
    packing::ParsePackedStruct,
    commands::Control,
};

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct WriteXCommand {
    pub lba: u32,
    pub transfer_length: u32,
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, Packed)]
#[packed(big_endian, lsb0)]
pub struct Write6Command {
    #[pkd(4, 0, 0, 2)]
    pub lba: u32,

    #[pkd(7, 0, 3, 3)]
    pub transfer_length: u8,

    #[pkd(7, 0, 4, 4)]
    pub control: Control,
}
impl ParsePackedStruct for Write6Command {}
impl From<Write6Command> for WriteXCommand {
    fn from(w: Write6Command) -> Self {
        Self {
            lba: w.lba.into(),
            transfer_length: w.transfer_length.into(),
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, Packed)]
#[packed(big_endian, lsb0)]
pub struct Write10Command {
    #[pkd(7, 5, 0, 0)]
    pub wr_protect: u8,

    #[pkd(4, 4, 0, 0)]
    pub dpo: bool,
    
    #[pkd(3, 3, 0, 0)]
    pub fua: bool,

    #[pkd(1, 1, 0, 0)]
    pub fua_nv: bool,

    #[pkd(7, 0, 1, 4)]
    pub lba: u32,

    #[pkd(4, 0, 5, 5)]
    pub group_number: u8,

    #[pkd(7, 0, 6, 7)]
    pub transfer_length: u16,

    #[pkd(7, 0, 8, 8)]
    pub control: Control,
}
impl ParsePackedStruct for Write10Command {}
impl From<Write10Command> for WriteXCommand {
    fn from(w: Write10Command) -> Self {
        Self {
            lba: w.lba.into(),
            transfer_length: w.transfer_length.into(),
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, Packed)]
#[packed(big_endian, lsb0)]
pub struct Write12Command {
    #[pkd(7, 5, 0, 0)]
    pub wr_protect: u8,

    #[pkd(4, 4, 0, 0)]
    pub dpo: bool,
    
    #[pkd(3, 3, 0, 0)]
    pub fua: bool,

    #[pkd(1, 1, 0, 0)]
    pub fua_nv: bool,

    #[pkd(7, 0, 1, 4)]
    pub lba: u32,

    #[pkd(7, 0, 5, 8)]
    pub transfer_length: u32,

    #[pkd(4, 0, 9, 9)]
    pub group_number: u8,

    #[pkd(7, 0, 10, 10)]
    pub control: Control,
}
impl ParsePackedStruct for Write12Command {}
impl From<Write12Command> for WriteXCommand {
    fn from(w: Write12Command) -> Self {
        Self {
            lba: w.lba.into(),
            transfer_length: w.transfer_length.into(),
        }
    }
}