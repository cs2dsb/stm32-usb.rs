use core::{
    mem::size_of,
    convert::Infallible,
};
use crate::{
    Packed,
    Endian,
    Bit,
    Unsigned, U1, U4,
    IsLessOrEqual,
    IsGreaterOrEqual,
};

impl<S: Bit, E: Bit, W: Unsigned> Packed<S, E, W> for u32 
where
    W: IsLessOrEqual<U4> + IsGreaterOrEqual<U1>
{
    type Error = Infallible;
    /// Number of bytes u32 packs/unpacks to/from (1-4)
    ///
    /// Note `W` type parameter influences the size for a given implementation
    const BYTES: usize = W::USIZE;
    // See the u16 impl for some extra comments on what's going on here
    fn unpack<En: Endian>(bytes: &[u8]) -> Result<Self, Self::Error> {
        assert!(bytes.len() == W::USIZE);

        let mut bytes_owned = [0; size_of::<Self>()];
        En::copy_bytes::<W>(bytes, &mut bytes_owned);

        En::align_bytes::<S,E>(&mut bytes_owned[..W::USIZE]);

        if En::IS_LITTLE {
            Ok(Self::from_le_bytes(bytes_owned))
        } else {
            Ok(Self::from_be_bytes(bytes_owned))
        }
    }     

    fn pack<En: Endian>(&self, bytes: &mut [u8]) -> Result<(), Self::Error> { 
        assert!(bytes.len() == size_of::<Self>());

        let mut field_bytes = if En::IS_LITTLE {
            self.to_le_bytes()
        } else {
            self.to_be_bytes()
        };

        En::unalign_bytes::<S,E>(&mut field_bytes);
        En::merge_field::<S, E, W>(&field_bytes, bytes);

        Ok(())
    } 
}