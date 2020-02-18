use core::{
    mem::size_of,
    convert::Infallible,
};
use crate::{
    Packed,
    Endian,
    Bit,
    Unsigned, U1, U2,
    IsLessOrEqual,
    IsGreaterOrEqual,
};

impl<S: Bit, E: Bit, W: Unsigned> Packed<S, E, W> for u16 
where
    W: IsLessOrEqual<U2> + IsGreaterOrEqual<U1>
{
    type Error = Infallible;
    /// Number of bytes u16 packs/unpacks to/from (1-2)
    ///
    /// Note `W` type parameter influences the size for a given implementation
    const BYTES: usize = W::USIZE;
    fn unpack<En: Endian>(bytes: &[u8]) -> Result<Self, Self::Error> {
        assert!(bytes.len() == W::USIZE);

        // I benchmarked using uninit or try_from instead of initializing the
        // array with 0s before overwriting and try_from/copy_from_slice were
        // the same and uninit was much slower.
        // Only benchmarked up to u32 and only on x86 so could revisit finding
        // the optimum initilization method for different architectures.
        // OTOH, it likely doesn't matter in the majority of cases.
        // Regardless, since we support W < size_of::<Self>() copy_from_slice
        // is ideal.
        let mut bytes_owned = [0; size_of::<Self>()];

        // Copy the bytes in an endian aware way
        En::copy_bytes::<W>(bytes, &mut bytes_owned);
        
        // Deal with non-zero start and end bits
        // ..W::USIZE is because if we're given a shorter array with non-zero
        // start and end bits they are referring to the data we've been given
        // not the full width type we've just created
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