use packing::{
    Packed,
    PackedSize,
};
use crate::scsi::commands::medium_type::MediumType;

#[derive(Clone, Copy, Eq, PartialEq, Debug, Packed)]
#[packed(big_endian, lsb0)]
pub struct ModeParameterHeader6 {
    #[packed(start_bit=7, end_bit=0, start_byte=0, end_byte=0)]
    pub mode_data_length: u8,

    #[packed(start_bit=7, end_bit=0, start_byte=1, end_byte=1)]
    pub medium_type: MediumType,

    #[packed(start_bit=7, end_bit=0, start_byte=2, end_byte=2)]
    pub device_specific_parameter: SbcDeviceSpecificParameter,

    #[packed(start_bit=7, end_bit=0, start_byte=3, end_byte=3)]
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
    pub fn increase_length_for_page(&mut self, page_code: PageCode) {
        self.mode_data_length += match page_code {
            PageCode::CachingModePage => CachingModePage::BYTES as u8,
        };
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, Packed)]
#[packed(big_endian, lsb0)]
pub struct ModeParameterHeader10 {
    #[packed(start_bit=7, end_bit=0, start_byte=0, end_byte=1)]
    pub mode_data_length: u16,

    #[packed(start_bit=7, end_bit=0, start_byte=2, end_byte=2)]
    pub medium_type: MediumType,

    #[packed(start_bit=7, end_bit=0, start_byte=3, end_byte=3)]
    pub device_specific_parameter: SbcDeviceSpecificParameter,

    #[packed(start_bit=0, end_bit=0, start_byte=4, end_byte=4)]
    pub long_lba: bool,

    #[packed(start_bit=7, end_bit=0, start_byte=6, end_byte=7)]
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
impl ModeParameterHeader10 {
    /// Increase the relevant length fields to indicate the provided page follows this header
    /// can be called multiple times but be aware of the max length allocated by CBW
    #[allow(dead_code)]
    pub fn increase_length_for_page(&mut self, page_code: PageCode) {
        self.mode_data_length += match page_code {
            PageCode::CachingModePage => CachingModePage::BYTES as u16,
        };
    }
}


#[derive(Clone, Copy, Eq, PartialEq, Debug, Packed, Default)]
#[packed(big_endian, lsb0)]
pub struct SbcDeviceSpecificParameter {
    #[packed(start_bit=7, end_bit=7, start_byte=0, end_byte=0)]
    pub write_protect: bool,

    #[packed(start_bit=4, end_bit=4, start_byte=0, end_byte=0)]
    pub disable_page_out_and_force_unit_access_available: bool,
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, Packed)]
pub enum PageCode {
    CachingModePage = 0x08,
}

/// This is only a partial implementation, there are a whole load of extra
/// fields defined in SBC-3 6.4.5
/// Default config is no read or write cache
#[derive(Clone, Copy, Eq, PartialEq, Debug, Packed)]
#[packed(big_endian, lsb0)]
pub struct CachingModePage {
    #[packed(start_bit=5, end_bit=0, start_byte=0, end_byte=0)]
    pub page_code: PageCode,

    #[packed(start_bit=7, end_bit=0, start_byte=1, end_byte=1)]
    pub page_length: u8,

    #[packed(start_bit=2, end_bit=2, start_byte=2, end_byte=2)]
    pub write_cache_enabled: bool,

    #[packed(start_bit=0, end_bit=0, start_byte=2, end_byte=2)]
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