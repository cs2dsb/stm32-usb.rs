use packing::{
    Packed,
    PackedSize,
};
use crate::scsi::{
    packing::ParsePackedStruct,
    commands::{
        Control,
        response_code::ResponseCode,
        sense_key::SenseKey,
        //additional_sense_code::AdditionalSenseCode,
    },
};

#[derive(Clone, Copy, Eq, PartialEq, Debug, Packed)]
#[packed(big_endian, lsb0)]
pub struct RequestSenseCommand {
    #[pkd(7, 0, 0, 0)]
    pub op_code: u8,
    
    #[pkd(0, 0, 1, 1)]
    pub descriptor_format: bool,
    
    #[pkd(7, 0, 4, 4)]
    pub allocation_length: u8,
    
    #[pkd(7, 0, 5, 5)]
    pub control: Control,
}
impl ParsePackedStruct for RequestSenseCommand {}


#[derive(Clone, Copy, Packed)]
#[packed(big_endian, lsb0)]
pub struct RequestSenseResponse {
    #[pkd(7, 0, 0, 0)]
    pub op_code: u8,
    
    #[pkd(7, 7, 1, 1)]
    pub valid: bool,
    
    #[pkd(6, 0, 1, 1)]
    pub response_code: ResponseCode,
    
    #[pkd(7, 7, 2, 2)]
    pub filemark: bool,
    
    #[pkd(6, 6, 2, 2)]
    pub end_of_medium: bool,
    
    #[pkd(5, 5, 2, 2)]
    pub incorrect_length_indicator: bool,
    
    #[pkd(3, 0, 2, 2)]
    pub sense_key: SenseKey,
    
    #[pkd(7, 0, 4, 7)]
    pub information: u32,
    
    #[pkd(7, 0, 8, 8)]
    /// n-7 
    pub additional_sense_length: u8,
    
    #[pkd(7, 0, 9, 12)]
    pub command_specifc_information: u32,
    
    #[pkd(7, 0, 13, 14)]
    pub additional_sense_code: u16,//AdditionalSenseCode,
    
    #[pkd(7, 0, 15, 15)]
    pub field_replaceable_unit_code: u8,
    
    #[pkd(7, 7, 16, 16)]
    pub sense_key_specific_valid: bool,
    
    #[pkd(6, 0, 16, 18)]
    pub sense_key_specific: u32,
    
    #[pkd(7, 0, 19, 253)]
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
            additional_sense_length: Self::BYTES as u8 - 7,
            sense_key_specific_valid: true,
            additional_sense_data: [0; 235],

            op_code: Default::default(),
            response_code: Default::default(),
            filemark: Default::default(),
            end_of_medium: Default::default(),
            incorrect_length_indicator: Default::default(),
            sense_key: Default::default(),
            information: Default::default(),
            command_specifc_information: Default::default(),
            additional_sense_code: Default::default(),
            field_replaceable_unit_code: Default::default(),
            sense_key_specific: Default::default(),
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