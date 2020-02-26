use crate::{
    Endian,
    PackedBytes,
};
use core::convert::Infallible;

impl PackedBytes<[u8; 8]> for u64 {
    type Error = Infallible;
    fn to_bytes<En: Endian>(&self) -> Result<[u8; 8], Self::Error> {
        Ok(if En::IS_LITTLE {
            self.to_le_bytes()
        } else {
            self.to_be_bytes()
        })
    }
    fn from_bytes<En: Endian>(bytes: [u8; 8]) -> Result<Self, Self::Error> {
        Ok(if En::IS_LITTLE {
            Self::from_le_bytes(bytes)
        } else {
            Self::from_be_bytes(bytes)
        })
    }
}