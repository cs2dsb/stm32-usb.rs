use packing::Packed;
use crate::scsi::{
    packing::ParsePackedStruct,
    commands::Control,
};

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct ReadXCommand {
    pub lba: u32,
    pub transfer_length: u32,
}


#[derive(Clone, Copy, Eq, PartialEq, Debug, Packed)]
#[packed(big_endian, lsb0)]
pub struct Read6Command {
    #[pkd(4, 0, 0, 2)]
    pub lba: u32,
    #[pkd(7, 0, 3, 3)]
    pub transfer_length: u8,
    #[pkd(7, 0, 4, 4)]
    pub control: Control,
}
impl ParsePackedStruct for Read6Command {}

impl From<Read6Command> for ReadXCommand {
    fn from(r: Read6Command) -> Self {
        Self {
            lba: r.lba.into(),
            transfer_length: r.transfer_length.into(),
        }
    }
}


#[derive(Clone, Copy, Eq, PartialEq, Debug, Packed)]
#[packed(big_endian, lsb0)]
pub struct Read10Command {
    #[pkd(7, 5, 0, 0)]
    pub rd_protect: u8,

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
impl ParsePackedStruct for Read10Command {}
impl From<Read10Command> for ReadXCommand {
    fn from(r: Read10Command) -> Self {
        Self {
            lba: r.lba.into(),
            transfer_length: r.transfer_length.into(),
        }
    }
}


#[derive(Clone, Copy, Eq, PartialEq, Debug, Packed)]
#[packed(big_endian, lsb0)]
pub struct Read12Command {
    #[pkd(7, 5, 0, 0)]
    pub rd_protect: u8,

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
impl ParsePackedStruct for Read12Command {}
impl From<Read12Command> for ReadXCommand {
    fn from(r: Read12Command) -> Self {
        Self {
            lba: r.lba.into(),
            transfer_length: r.transfer_length.into(),
        }
    }
}




#[test]
fn test_read10_parse() {
    let data = [0, 0, 0, 0x1E, 0x80, 0, 0, 0x8, 0, 0, 0, 0, 0, 0, 0];
    let cmd = Read10Command::parse(&data).unwrap();
    assert_eq!(cmd.lba, 0x1E80);
}