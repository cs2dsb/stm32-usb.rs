use packed_struct_codegen::PackedStruct;
use packed_struct::PackedStruct;
use crate::scsi::{
    packing::{ ResizeSmaller, ParsePackedStruct },
    commands::{
        Control,
        CommandLength,
        page_control::PageControl,
    },
};


/* After a logical unit reset, the device server shall respond in the following manner:
a) if default values are requested, report the default values;
b) if saved values are requested, report valid restored mode parameters, or restore the mode parameters and
report them. If the saved values of the mode parameters are not able to be accessed from the nonvolatile
vendor specific location, the command shall be terminated with CHECK CONDITION status, with the
sense key set to NOT READY. If saved parameters are not implemented, respond as defined in 6.11.5; or
c) if current values are requested and the current values have been sent by the application client via a MODE
SELECT command, the current values shall be returned. If the current values have not been sent, the
device server shall return:
A) the saved values, if saving is implemented and saved values are available; or
B) the default values.
*/

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct ModeSenseXCommand {
    pub command_length: CommandLength,
    pub page_control: PageControl
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, PackedStruct)]
#[packed_struct(endian="msb", bit_numbering="msb0")]
pub struct ModeSense6Command {
    #[packed_field(bits="3..4")]
    pub disable_block_descriptors: bool,
    #[packed_field(bits="8..10", ty="enum")]
    pub page_control: PageControl,
    #[packed_field(bits="10..16")]
    pub page_code: u8,
    #[packed_field(bits="16..24")]
    pub subpage_code: u8,
    #[packed_field(bits="24..32")]
    pub allocation_length: u8,
    #[packed_field(bits="32..40")]
    pub control: Control,
}
impl<A: ResizeSmaller<[u8; ModeSense6Command::BYTES]>> ParsePackedStruct<A, [u8; ModeSense6Command::BYTES]> for ModeSense6Command {}
impl From<ModeSense6Command> for ModeSenseXCommand {
    fn from(m: ModeSense6Command) -> Self {
        Self { 
            command_length: CommandLength::C6,
            page_control: m.page_control,
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, PackedStruct)]
#[packed_struct(endian="msb", bit_numbering="msb0")]
pub struct ModeSense10Command {
    #[packed_field(bits="3..4")]
    pub long_lba_accepted: bool,
    #[packed_field(bits="4..5")]
    pub disable_block_descriptors: bool,
    #[packed_field(bits="8..10", ty="enum")]
    pub page_control: PageControl,
    #[packed_field(bits="10..16")]
    pub page_code: u8,
    #[packed_field(bits="16..24")]
    pub subpage_code: u8,
    #[packed_field(bits="48..64")]
    pub allocation_length: u16,
    #[packed_field(bits="64..72")]
    pub control: Control,
}
impl<A: ResizeSmaller<[u8; ModeSense10Command::BYTES]>> ParsePackedStruct<A, [u8; ModeSense10Command::BYTES]> for ModeSense10Command {}
impl From<ModeSense10Command> for ModeSenseXCommand {
    fn from(m: ModeSense10Command) -> Self {
        Self {
            command_length: CommandLength::C10,
            page_control: m.page_control, 
        }
    }
}