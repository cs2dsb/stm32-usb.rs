use core::{
    mem::size_of,
    convert::Infallible,
};

use crate::{
    Packed,
    Endian,
    Bit,
    U1, IsLessOrEqual, Unsigned, IsGreaterOrEqual,
};

impl<S: Bit, E: Bit, W: Unsigned> Packed<S, E, W> for u8 
where 
    E: IsLessOrEqual<S>,
    W: IsLessOrEqual<U1> + IsGreaterOrEqual<U1>
{
    type Error = Infallible;

    /// Number of bytes u8 packs/unpacks to/from (1)
    const BYTES: usize = W::USIZE;
    fn unpack<En: Endian>(bytes: &[u8]) -> Result<Self, Self::Error> {
        assert!(bytes.len() == W::USIZE);

        let mut bytes_owned = [0; size_of::<Self>()];
        En::copy_bytes::<W>(bytes, &mut bytes_owned);

        En::align_bytes::<S,E>(&mut bytes_owned);

        if En::IS_LITTLE {
            Ok(u8::from_le_bytes(bytes_owned))
        } else {
            Ok(u8::from_be_bytes(bytes_owned))
        }
    }

    fn pack<En: Endian>(&self, bytes: &mut [u8]) -> Result<(), Self::Error> { 
        assert!(bytes.len() == W::USIZE);

        let mut field_bytes = if En::IS_LITTLE {
            self.to_le_bytes()
        } else {
            self.to_be_bytes()
        };

        En::unalign_bytes::<S,E>(&mut field_bytes);
        En::merge_field::<S, E, U1>(&field_bytes, bytes);

        Ok(())
    } 
}

#[cfg(test)]
mod tests {
    use crate::*;

    fn pack_unpack<S: Bit, E: Bit + IsLessOrEqual<S>>(v: u8) {
        let packed = (v << E::USIZE) & S::HEAD_MASK;
        let target = packed >> E::USIZE;
        let mut b = [packed];

        let u = <u8 as Packed<S, E, U1>>::unpack::<LittleEndian>(&b).unwrap();
        assert_eq!(target, u);

        <u8 as Packed<S, E, U1>>::pack::<LittleEndian>(&u, &mut b).unwrap();
        assert_eq!(b, [packed]);
    }

    #[test]
    fn test_u8() {
        for i in 0..=u8::max_value() {
            pack_unpack::<U0, U0>(i);

            pack_unpack::<U1, U0>(i);
            pack_unpack::<U2, U1>(i);
            pack_unpack::<U3, U2>(i);
            pack_unpack::<U4, U3>(i);
            pack_unpack::<U5, U4>(i);
            pack_unpack::<U6, U5>(i);
            pack_unpack::<U7, U6>(i);

            pack_unpack::<U2, U0>(i);
            pack_unpack::<U3, U1>(i);
            pack_unpack::<U4, U2>(i);
            pack_unpack::<U5, U3>(i);
            pack_unpack::<U6, U4>(i);
            pack_unpack::<U7, U5>(i);

            pack_unpack::<U3, U0>(i);
            pack_unpack::<U4, U1>(i);
            pack_unpack::<U5, U2>(i);
            pack_unpack::<U6, U3>(i);
            pack_unpack::<U7, U4>(i);

            pack_unpack::<U4, U0>(i);
            pack_unpack::<U5, U1>(i);
            pack_unpack::<U6, U2>(i);
            pack_unpack::<U7, U3>(i);

            pack_unpack::<U5, U0>(i);
            pack_unpack::<U6, U1>(i);
            pack_unpack::<U7, U2>(i);


            pack_unpack::<U6, U0>(i);
            pack_unpack::<U7, U1>(i);


            pack_unpack::<U7, U0>(i);
        }
    }
}