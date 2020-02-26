use crate::{
    Endian,
    PackedBytes,
};
use core::convert::Infallible;

impl PackedBytes<[u8; 1]> for bool {
    type Error = Infallible;
    fn to_bytes<En: Endian>(&self) -> Result<[u8; 1], Self::Error> {
        Ok(if *self {
           [1]
        } else {
           [0]
        })
    }
    fn from_bytes<En: Endian>(bytes: [u8; 1]) -> Result<Self, Self::Error> {
        Ok(bytes[0] == 1)
    }
}