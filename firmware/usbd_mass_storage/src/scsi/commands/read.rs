use packed_struct_codegen::PackedStruct;
use packed_struct::PackedStruct;
use crate::scsi::{
    packing::{ ResizeSmaller, ParsePackedStruct },
    commands::Control,
};

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct ReadXCommand {
    pub lba: u32,
    pub transfer_length: u32,
}


#[derive(Clone, Copy, Eq, PartialEq, Debug, PackedStruct)]
#[packed_struct(endian="msb", bit_numbering="msb0")]
pub struct Read6Command {
    #[packed_field(bits="3..=23")]
    pub lba: u32,
    #[packed_field(bits="24..=31")]
    pub transfer_length: u8,
    #[packed_field(bits="32..=39")]
    pub control: Control,
}
impl<A: ResizeSmaller<[u8; Read6Command::BYTES]>>  ParsePackedStruct<A, [u8; Read6Command::BYTES]> for Read6Command {}
impl From<Read6Command> for ReadXCommand {
    fn from(r: Read6Command) -> Self {
        Self {
            lba: r.lba.into(),
            transfer_length: r.transfer_length.into(),
        }
    }
}


#[derive(Clone, Copy, Eq, PartialEq, Debug, PackedStruct)]
#[packed_struct(endian="msb", bit_numbering="msb0")]
pub struct Read10Command {
    #[packed_field(bits="0..3")]
    pub rd_protect: u8,
    #[packed_field(bits="3")]
    pub dpo: bool,
    #[packed_field(bits="4")]
    pub fua: bool,
    #[packed_field(bits="5")]
    pub rarc: bool,
    #[packed_field(bits="8..40")]
    pub lba: u32,
    #[packed_field(bits="43..48")]
    pub group_number: u8, 
    #[packed_field(bits="48..64")]
    pub transfer_length: u16,
    #[packed_field(bits="64..72")]
    pub control: Control,
}
impl<A: ResizeSmaller<[u8; Read10Command::BYTES]>>  ParsePackedStruct<A, [u8; Read10Command::BYTES]> for Read10Command {}
impl From<Read10Command> for ReadXCommand {
    fn from(r: Read10Command) -> Self {
        Self {
            lba: r.lba.into(),
            transfer_length: r.transfer_length.into(),
        }
    }
}


#[derive(Clone, Copy, Eq, PartialEq, Debug, PackedStruct)]
#[packed_struct(endian="msb", bit_numbering="msb0")]
pub struct Read12Command {
    #[packed_field(bits="0..3")]
    pub rd_protect: u8,
    #[packed_field(bits="3..4")]
    pub dpo: bool,
    #[packed_field(bits="4..5")]
    pub fua: bool,
    #[packed_field(bits="5..6")]
    pub rarc: bool,
    #[packed_field(bits="8..40")]
    pub lba: u32,
    #[packed_field(bits="40..72")]
    pub transfer_length: u32,
    #[packed_field(bits="74..79")]
    pub group_number: u8,
    #[packed_field(bits="79..87")]
    pub control: Control
}
impl<A: ResizeSmaller<[u8; Read12Command::BYTES]>>  ParsePackedStruct<A, [u8; Read12Command::BYTES]> for Read12Command {}
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