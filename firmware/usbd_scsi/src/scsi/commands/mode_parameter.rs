use packed_struct_codegen::{ PackedStruct, PrimitiveEnum };
use packed_struct::PackedStruct;
use crate::scsi::commands::medium_type::MediumType;

#[derive(Clone, Copy, Eq, PartialEq, Debug, PackedStruct)]
#[packed_struct(endian="msb", bit_numbering="msb0")]
pub struct ModeParameterHeader6 {
    pub mode_data_length: u8,
    #[packed_field(bits="8..16", ty="enum")]
    pub medium_type: MediumType,
    #[packed_field(bits="16..24")]
    pub device_specific_parameter: SbcDeviceSpecificParameter,
    pub block_descriptor_length: u8,
}
impl Default for ModeParameterHeader6 {
    fn default() -> Self {
        Self {
            mode_data_length: Self::BYTES as u8 - 1,
            medium_type: Default::default(),
            device_specific_parameter: Default::default(),
            block_descriptor_length: 0,
        }
    }
}
impl ModeParameterHeader6 {
    /// Increase the relevant length fields to indicate the provided page follows this header
    /// can be called multiple times but be aware of the max length allocated by CBW
    pub fn increase_length_for_page(&mut self, page: &[u8]) {
        self.mode_data_length += page.len() as u8;
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, PackedStruct)]
#[packed_struct(endian="msb", bit_numbering="msb0")]
pub struct ModeParameterHeader10 {
    pub mode_data_length: u16,
    #[packed_field(bits="16..24", ty="enum")]
    pub medium_type: MediumType,
    #[packed_field(bits="24..32")]
    pub device_specific_parameter: SbcDeviceSpecificParameter,
    #[packed_field(bits="39..40")]
    pub long_lba: bool,
    #[packed_field(bits="48..64")]
    pub block_descriptor_length: u16,
}
impl Default for ModeParameterHeader10 {
    fn default() -> Self {
        Self {
            mode_data_length: Self::BYTES as u16 - 2,
            medium_type: Default::default(),
            device_specific_parameter: Default::default(),
            long_lba: Default::default(),
            block_descriptor_length: 0,
        }
    }
}
/*
impl ModeParameterHeader10 {
    /// Increase the relevant length fields to indicate the provided page follows this header
    /// can be called multiple times but be aware of the max length allocated by CBW
    pub fn increase_length_for_page(&mut self, page: &[u8]) {
        self.mode_data_length += page.len() as u16;
    }
}
*/


#[derive(Clone, Copy, Eq, PartialEq, Debug, PackedStruct, Default)]
#[packed_struct(endian="msb", bit_numbering="msb0")]
pub struct SbcDeviceSpecificParameter {
    #[packed_field(bits="0..1")]
    pub write_protect: bool,
    #[packed_field(bits="3..4")]
    pub disable_page_out_and_force_unit_access_available: bool,
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, PrimitiveEnum)]
pub enum PageCode {
    CachingModePage = 0x08,
}

/// This is only a partial implementation, there are a whole load of extra
/// fields defined in SBC-3 6.4.5
/// Default config is no read or write cache
#[derive(Clone, Copy, Eq, PartialEq, Debug, PackedStruct)]
#[packed_struct(endian="msb", bit_numbering="msb0", size_bytes="19")]
pub struct CachingModePage {
    #[packed_field(bits="2..8", ty="enum")]
    pub page_code: PageCode,
    #[packed_field(bits="8..16")]
    pub page_length: u8,
    #[packed_field(bits="21..22")]
    pub write_cache_enabled: bool,
    #[packed_field(bits="23..24")]
    pub read_cache_disable: bool,
}
impl Default for CachingModePage {
    fn default() -> Self {
        Self {
            page_code: PageCode::CachingModePage,
            page_length: Self::BYTES as u8,
            write_cache_enabled: false,
            read_cache_disable: true,
        }
    }
}