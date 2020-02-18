use core::{
    mem::size_of,
    convert::Infallible,
};
use crate::{
    Packed,
    Endian,
    Bit,
    U1,
};

impl<B: Bit> Packed<B, B, U1> for bool 
{
    type Error = Infallible;
    /// Number of bytes bool packs/unpacks to/from (1)
    ///
    /// Note `W` type parameter influences the size for a given implementation
    const BYTES: usize = size_of::<Self>();
    // See the u16 impl for some extra comments on what's going on here
    fn unpack<En: Endian>(bytes: &[u8]) -> Result<Self, Self::Error> {
        assert!(bytes.len() == 1);
        Ok(bytes[0] & B::BIT_MASK == B::BIT_MASK)
    }     

    fn pack<En: Endian>(&self, bytes: &mut [u8]) -> Result<(), Self::Error> { 
        assert!(bytes.len() == size_of::<Self>());

        if *self {
            bytes[0] |= B::BIT_MASK;
        } else {
            bytes[0] &= !B::BIT_MASK;
        }

        Ok(())
    } 
}