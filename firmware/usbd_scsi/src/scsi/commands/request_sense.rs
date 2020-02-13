use packed_struct_codegen::PackedStruct;
use packed_struct::PackedStruct;
use crate::scsi::{
    packing::{ ResizeSmaller, ParsePackedStruct },
    commands::{
        Control,
        response_code::ResponseCode,
        sense_key::SenseKey,
        additional_sense_code::AdditionalSenseCode,
    },
};

#[derive(Clone, Copy, Eq, PartialEq, Debug, PackedStruct)]
#[packed_struct(endian="msb", bit_numbering="msb0")]
pub struct RequestSenseCommand {
    #[packed_field(bits="7..8")]
    pub descriptor_format: bool,
    #[packed_field(bits="24..32")]
    pub allocation_length: u8,
    #[packed_field(bits="32..40")]
    pub control: Control,
}
impl<A: ResizeSmaller<[u8; RequestSenseCommand::BYTES]>> ParsePackedStruct<A, [u8; RequestSenseCommand::BYTES]> for RequestSenseCommand {}


#[derive(Clone, Copy, PackedStruct)]
#[packed_struct(endian="msb", bit_numbering="msb0")]
pub struct RequestSenseResponse {
    #[packed_field(bits="0..1")] 
    pub valid: bool,
    #[packed_field(bits="1..8", ty="enum")] 
    pub response_code: ResponseCode,
    #[packed_field(bits="16..17")] 
    pub filemark: bool,
    #[packed_field(bits="17..18")] 
    pub end_of_medium: bool,
    #[packed_field(bits="18..19")] 
    pub incorrect_length_indicator: bool,
    #[packed_field(bits="20..24", ty="enum")] 
    pub sense_key: SenseKey,
    #[packed_field(bits="24..56")] 
    pub information: u32,
    #[packed_field(bits="56..64")]
    /// n-7 
    pub additional_sense_length: u8,
    #[packed_field(bits="64..96")] 
    pub command_specifc_information: u32,
    #[packed_field(bits="96..112", ty="enum")] 
    pub additional_sense_code: AdditionalSenseCode,
    #[packed_field(bits="112..120")] 
    pub field_replaceable_unit_code: u8,
    #[packed_field(bits="120..121")] 
    pub sense_key_specific_valid: bool,
    #[packed_field(bits="121..144")] 
    pub sense_key_specific: u32,
    #[packed_field(bits="144..2024")] 
    pub additional_sense_data: [u8; 235],
}

/*
information
command_specifc_information
additional_sense_code
additional_sense_code_qualifier
sense_key_specific
*/

impl Default for RequestSenseResponse {
    fn default() -> Self {
        Self {
            valid: true,
            response_code: Default::default(),
            filemark: Default::default(),
            end_of_medium: Default::default(),
            incorrect_length_indicator: Default::default(),
            sense_key: Default::default(),
            information: Default::default(),
            additional_sense_length: Self::BYTES as u8 - 7,
            command_specifc_information: Default::default(),
            additional_sense_code: Default::default(),
            field_replaceable_unit_code: Default::default(),
            sense_key_specific_valid: true,
            sense_key_specific: Default::default(),
            additional_sense_data: [0; 235],
        }
    }
}

/*
    if !descriptor_format
        return fixed sense data
    else
        if descriptor sense data supported
            return descriptor sense data
        else 
            return CHECK CONDITION with sense:
                key: ILLEGAL REQUEST
                additional code: INVALID FIELD IN CDB

*/