use packed_struct_codegen::PrimitiveEnum;
use packing::{
    Packed,
    Error,
    U7, U0, U1,
    Endian,
};

/// The direction of a data transfer
#[derive(Clone, Copy, Eq, PartialEq, Debug, PrimitiveEnum)]
pub enum Direction {
    /// Host to device, OUT in USB parlance
    HostToDevice = 0x00,
    /// Device to host, IN in USB parlance
    DeviceToHost = 0x80,
}

impl Packed<U7, U0, U1> for Direction {
    type Error = Error;

    const BYTES: usize = 1;
    fn unpack<En: Endian>(bytes: &[u8]) -> Result<Self, Self::Error> {
        assert!(bytes.len() == 1);

        match bytes[0] {
            0x00 => Ok(Direction::HostToDevice),
            0x80 => Ok(Direction::DeviceToHost),
            _ => Err(Error::InvalidEnumDiscriminant),
        }
    }

    fn pack<En: Endian>(&self, bytes: &mut [u8]) -> Result<(), Self::Error> { 
        assert!(bytes.len() == 1);
        bytes[0] = match self {
            Direction::HostToDevice => 0x00,
            Direction::DeviceToHost => 0x80,
        };
        Ok(())
    } 
}
