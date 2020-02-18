#![cfg_attr(not(test), no_std)]

use core::{
    mem::size_of,
    convert::TryFrom,
};

pub use typenum::{
    Unsigned, U0, U1, U2, U3, U4, U5, U6, U7, U8, U9, U10, U11, U12, U13, U14, U15, U16, 
    IsLess, IsLessOrEqual, IsGreaterOrEqual, Cmp,
};

/// Derive for [Packed](trait.Packed.html)
pub use packing_codegen::Packed;


/// Enum of possible errors returned from packing functions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// Pack or Unpack method called with a slice of insufficient length
    /// Check the `PACK_BYTES_LEN` on the struct impl 
    InsufficientBytes,
}


/// Trait that covers functionality required to deal with non aligned endian sensitive fields
/// in packed structs
///
/// For example, 10 bits LE offset by 2:
///
/// | byte | 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |
/// |------|---|---|---|---|---|---|---|---|  
/// | 0    | 0 | 0 | F | E | D | C | B | A |
/// | 1    | J | I | H | G | 0 | 0 | 0 | 0 |
///
/// Should become:
///
/// | byte | 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0 |
/// |------|---|---|---|---|---|---|---|---|  
/// | 0    | H | G | F | E | D | C | B | A | 
/// | 1    | 0 | 0 | 0 | 0 | 0 | 0 | J | I |
///
pub trait Endian {
    /// Shift the bits in the byte slice to be 8 bit aligned in the endianness represented by 
    /// the implementing type
    ///
    /// Also masks away bytes outside the range specified by `S` and `E`.
    /// Does nothing if `S` == 7 and `E` == 0.
    ///
    /// `S` and `E` type parameters represent bit positions with 7 being the most significant bit and
    /// 0 being the least significant bit. `S` is the first included bit in the first byte of the slice.
    /// `E` is the last included bit in the last byte of the slice.
    ///
    /// Panics if `bytes`.len() == 0 or if `bytes`.len() == 1 AND S < E 
    fn shift_bytes<S: Bit, E: Bit>(bytes: &mut [u8]);

    /// Copies `W` bytes from `src` to `dest` in an endian aware way. Uses `copy_from_slice` (memcpy)
    ///
    /// i.e. little endian will copy the bytes to the front, big endian will copy the bytes to the back. 
    ///
    /// Panics if `src`.len() != `W::USIZE` or `dest`.len() < `W::USIZE`
    fn copy_bytes<W: Unsigned>(src: &[u8], dest: &mut [u8])
    where
        W: IsGreaterOrEqual<U1>;
}

/// Defines little endian bit shifting required for packing and unpacking non aligned fields in byte slices
///
/// Not construcatable, used only at type level
pub enum LittleEndian {}
impl Endian for LittleEndian {
    /// Shifts bytes towards the top of the slice, copying the LSB from the next byte if there is space
    /// in the previous byte
    ///
    /// Resulting bytes should be ready to be passed to <unsigned_primitive>::from_le_bytes(`bytes`)
    ///
    /// Also masks away any bytes outside the range specified by `S` and `E`.
    /// Does nothing if `S` == 7 and `E` == 0.
    ///
    /// `S` and `E` type parameters represent bit positions with 7 being the most significant bit and
    /// 0 being the least significant bit. `S` is the first included bit in the first byte of the slice.
    /// `E` is the last included bit in the last byte of the slice.
    ///
    /// Panics if `bytes`.len() == 0 or if `bytes`.len() == 1 AND S < E 
    fn shift_bytes<S: Bit, E: Bit>(bytes: &mut [u8]) {
        let len = bytes.len();
        
        // Not valid to call this with no data
        assert!(len > 0);
        // Not valid to call with 1 byte and S E bits overlapping
        assert!(len > 1 || S::USIZE >= E::USIZE);

        // If start and end are both at their respective byte boundaries there's nothing to do here
        if S::USIZE == 7 && E::USIZE == 0 { return; }

        // Mask away whatever is before the first bit
        bytes[0] &= S::HEAD_MASK;
        
        // Shift the final bits into the LSB. This also makes 
        // sure everything else in the byte is zero
        bytes[len-1] >>= E::USIZE;
        
        // If there is space in the first byte, shuffle everything up
        if S::USIZE != 7 {
            for i in 0..(len - 1) {
                let j = i + 1;
                // Add the number of bits we have space for from the next byte
                // No special checking for last byte because we already shifted it above
                bytes[i] |= bytes[j] << (S::USIZE + 1);

                // Shift what's left to the LSB
                bytes[j] >>= 7-S::USIZE;
            }
        }
    }

    /// Copies `W` bytes from `src` to the front of `dest`. Uses `copy_from_slice` (memcpy)
    ///
    /// Panics if `src`.len() != `W::USIZE` or `dest`.len() < `W::USIZE`
    fn copy_bytes<W: Unsigned>(src: &[u8], dest: &mut [u8])
    where
        W: IsGreaterOrEqual<U1> 
    {
        assert!(src.len() == W::USIZE);
        assert!(dest.len() >= W::USIZE);

        dest[..W::USIZE].copy_from_slice(src);
    }
}

/// Defines big endian bit shifting required for packing and unpacking non aligned fields in byte slices
///
/// Not construcatable, used only at type level
pub enum BigEndian {}
impl Endian for BigEndian {
    /// Shifts bytes towards the bottom of the slice, copying the MSB from the previous byte if there is space
    /// in the next byte
    ///
    /// Resulting bytes should be ready to be passed to <unsigned_primitive>::from_be_bytes(`bytes`)
    ///
    /// Also masks away any bytes outside the range specified by `S` and `E`.
    /// Does nothing if `S` == 7 and `E` == 0.
    ///
    /// `S` and `E` type parameters represent bit positions with 7 being the most significant bit and
    /// 0 being the least significant bit. `S` is the first included bit in the first byte of the slice.
    /// `E` is the last included bit in the last byte of the slice.
    ///
    /// Panics if `bytes`.len() == 0 or if `bytes`.len() == 1 AND S < E 
    fn shift_bytes<S: Bit, E: Bit>(bytes: &mut [u8]) {
        let len = bytes.len();
        
        // Not valid to call this with no data
        assert!(len > 0);
        // Not valid to call with 1 byte and S E bits overlapping
        assert!(len > 1 || S::USIZE >= E::USIZE);

        // If start and end are both at their respective byte boundaries there's nothing to do here
        if S::USIZE == 7 && E::USIZE == 0 { return; }

        // Mask away whatever is before the first bit
        bytes[0] &= S::HEAD_MASK;
        
        // Shift the final bits into the LSB. This also makes 
        // sure everything else in the byte is zero
        bytes[len-1] >>= E::USIZE;
        
        // If there is space in the last byte, shuffle everything down
        if E::USIZE != 7 {
            for i in (1..len).rev() {
                let j = i - 1;
                // Add the number of bits we have space for from the next byte
                // No special checking for last byte because we already shifted it above
                bytes[i] |= bytes[j] << (7-E::USIZE);
                // Shift what's left to the LSB
                bytes[j] >>= E::USIZE;
            }
        }
    }

    /// Copies `W` bytes from `src` to the end of `dest`. Uses `copy_from_slice` (memcpy)
    ///
    /// Panics if `src`.len() != `W::USIZE` or `dest`.len() < `W::USIZE`
    fn copy_bytes<W: Unsigned>(src: &[u8], dest: &mut [u8])
    where
        W: IsGreaterOrEqual<U1> 
    {
        assert!(src.len() == W::USIZE);
        assert!(dest.len() >= W::USIZE);

        let s = dest.len() - W::USIZE;
        dest[s..].copy_from_slice(src);
    }
}

/// Trait signifying a single bit in a byte. 7 = most significant bit, 0 = least significant bit
pub trait Bit: IsLess<U8> + Unsigned {
    /// The mask used to discard bits before this bit (i.e. if this bit is 5, ORing mask with a u8
    /// will ensure bits 7 and 6 are 0.
    const HEAD_MASK: u8 = ((1_u16 << (Self::USIZE + 1)) - 1) as u8;
    /// The mask used to extract the single bit from a byte
    const BIT_MASK: u8 = 1 << Self::USIZE;
}

impl Bit for U0 { }
impl Bit for U1 { }
impl Bit for U2 { }
impl Bit for U3 { }
impl Bit for U4 { }
impl Bit for U5 { }
impl Bit for U6 { }
impl Bit for U7 { }


/// Trait that provides packing and unpacking of arbitrarily aligned endian aware fields
/// to/from byte slices
pub trait Packed<B, S: Bit, E: Bit, W: Unsigned> where Self: Sized {
    /// Number of bytes implementation packs/unpacks to/from
    ///
    /// Note `W` type parameter influences the size for a given implementation. For example, a
    /// `u32` can be packed or unpacked from 1, 2, 3 or 4 bytes
    const PACK_BYTES_LEN: usize;

    /// Unpacks provided bytes into `Self` in an endian aware way. Typically `bytes` will be
    /// &[u8] to allow slices of a shared buffer bo be used with this type but could also
    /// be implemented for fixed size arrays if that is useful.
    ///
    /// In the slice case, most implementations allocate a new fixed sized array and use 
    /// `copy_from_slice` (memcpy) to copy the bytes from the slice into it before performing
    /// the field shifting and conversion.
    /// For primitive types, this doesn't result in any wasted allocation - the new fixed sized
    /// array is directly transmuted into the new type and endian conversion is done in place
    /// using an intrinsic.
    ///
    /// I briefly benchmarked using [core::mem::MaybeUninit] against [core::convert::TryFrom::try_from] 
    /// and [core::slice::copy_from_slice] and MaybeUninit was substantially slower which try_from and 
    /// [0; LEN].copy_from_slice were the same speed. copy_from_slice was favoured because it works with
    /// input bytes less than the total size of the type without any extra allocation - so for example 
    /// the bytes for a u24 can be copied to the front or back of the 4 bytes for a u32
    fn unpack<En: Endian>(bytes: B) -> Result<Self, Error>;

    /// Packs self into provided mutable reference in an endian aware way. Typically `bytes` will
    /// be &mut [u8] to allow slices of shared buffer to be used with this type but could also be
    /// implemented for fixed size arrays if that is useful.
    ///
    /// In general, implementations use to_le_bytes or to_be_bytes to generate the correct byte 
    /// represenations from primitive types which will allocate small temporary arrays on the stack.
    /// Benchmarking showed that using to_x_bytes was as fast or faster in most cases than hand written
    /// bit shifting. Some shifting is still required if S or E indicate a non byte aligned field.
    fn pack<En: Endian>(&self, bytes: &mut B) -> Result<(), Error> { let _ = bytes; unimplemented!(); }
}

impl<S: Bit, E: Bit> Packed<&[u8], S, E, U1> for u8 
where 
    E: IsLessOrEqual<S>
{
    /// Number of bytes u8 packs/unpacks to/from (1)
    const PACK_BYTES_LEN: usize = size_of::<Self>();
    fn unpack<En: Endian>(bytes: &[u8]) -> Result<Self, Error> {
        assert!(bytes.len() == size_of::<Self>());

        let mut bytes = <[u8; size_of::<Self>()]>::try_from(bytes).unwrap();
        En::shift_bytes::<S,E>(&mut bytes);

        Ok(u8::from_le_bytes(bytes))        
    }
}

impl<S: Bit, E: Bit, W: Unsigned> Packed<&[u8], S, E, W> for u16 
where
    W: IsLessOrEqual<U2> + IsGreaterOrEqual<U1>
{
    /// Number of bytes u16 packs/unpacks to/from (1-2)
    ///
    /// Note `W` type parameter influences the size for a given implementation
    const PACK_BYTES_LEN: usize = W::USIZE;
    fn unpack<En: Endian>(bytes: &[u8]) -> Result<Self, Error> {
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
        En::shift_bytes::<S,E>(&mut bytes_owned[..W::USIZE]);

        Ok(u16::from_le_bytes(bytes_owned))
    } 
}

impl<S: Bit, E: Bit, W: Unsigned> Packed<&[u8], S, E, W> for u32 
where
    W: IsLessOrEqual<U4> + IsGreaterOrEqual<U1>
{
    /// Number of bytes u32 packs/unpacks to/from (1-4)
    ///
    /// Note `W` type parameter influences the size for a given implementation
    const PACK_BYTES_LEN: usize = W::USIZE;
    // See the u16 impl for some extra comments on what's going on here
    fn unpack<En: Endian>(bytes: &[u8]) -> Result<Self, Error> {
        assert!(bytes.len() == W::USIZE);

        let mut bytes_owned = [0; size_of::<Self>()];
        En::copy_bytes::<W>(bytes, &mut bytes_owned);

        En::shift_bytes::<S,E>(&mut bytes_owned[..W::USIZE]);

        Ok(u32::from_le_bytes(bytes_owned))
    } 
}

impl<S: Bit, E: Bit, W: Unsigned> Packed<&[u8], S, E, W> for u64 
where
    W: IsLessOrEqual<U8> + IsGreaterOrEqual<U1>
{
    /// Number of bytes u64 packs/unpacks to/from (1-8)
    ///
    /// Note `W` type parameter influences the size for a given implementation
    const PACK_BYTES_LEN: usize = W::USIZE;
    // See the u16 impl for some extra comments on what's going on here
    fn unpack<En: Endian>(bytes: &[u8]) -> Result<Self, Error> {
        assert!(bytes.len() == W::USIZE);

        let mut bytes_owned = [0; size_of::<Self>()];
        En::copy_bytes::<W>(bytes, &mut bytes_owned);

        En::shift_bytes::<S,E>(&mut bytes_owned[..W::USIZE]);
        
        Ok(u64::from_le_bytes(bytes_owned))
    } 
}

impl<S: Bit, E: Bit, W: Unsigned> Packed<&[u8], S, E, W> for u128 
where
    W: IsLessOrEqual<U16> + IsGreaterOrEqual<U1>
{
    /// Number of bytes u128 packs/unpacks to/from (1-16)
    ///
    /// Note `W` type parameter influences the size for a given implementation
    const PACK_BYTES_LEN: usize = W::USIZE;
    // See the u16 impl for some extra comments on what's going on here
    fn unpack<En: Endian>(bytes: &[u8]) -> Result<Self, Error> {
        assert!(bytes.len() == W::USIZE);

        let mut bytes_owned = [0; size_of::<Self>()];
        En::copy_bytes::<W>(bytes, &mut bytes_owned);

        En::shift_bytes::<S,E>(&mut bytes_owned[..W::USIZE]);
        
        Ok(u128::from_le_bytes(bytes_owned))
    } 
}

impl<B: Bit> Packed<&[u8], B, B, U1> for bool 
{
    /// Number of bytes bool packs/unpacks to/from (1)
    ///
    /// Note `W` type parameter influences the size for a given implementation
    const PACK_BYTES_LEN: usize = size_of::<Self>();
    // See the u16 impl for some extra comments on what's going on here
    fn unpack<En: Endian>(bytes: &[u8]) -> Result<Self, Error> {
        assert!(bytes.len() == 1);
        Ok(bytes[0] & B::BIT_MASK == B::BIT_MASK)
    } 
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shift_le() {
        let mut bytes1 = [
            0b00011111,
            0b11111111,
            0b11111000,
        ];
        let mut bytes2 = [
            0b11111111,
            0b11111111,
            0b11111111,
        ];
        let target = [
            0b11111111,
            0b11111111,
            0b00000011,
        ];

        LittleEndian::shift_bytes::<U4, U3>(&mut bytes1);
        LittleEndian::shift_bytes::<U4, U3>(&mut bytes2);

        assert_eq!(bytes1, target);
        assert_eq!(bytes2, target);

        let mut bytes1 = [
            0b00011111,
            0b11101111,
            0b11111000,
        ];
        let mut bytes2 = [
            0b11111111,
            0b11101111,
            0b11111111,
        ];
        let target = [
            0b11111111,
            0b11111101,
            0b00000011,
        ];

        LittleEndian::shift_bytes::<U4, U3>(&mut bytes1);
        LittleEndian::shift_bytes::<U4, U3>(&mut bytes2);

        //panic!("\n0b{:08b}\n0b{:08b}\n0b{:08b}", bytes1[0], bytes1[1], bytes1[2]);
        assert_eq!(bytes1, target);
        assert_eq!(bytes2, target);
    }

    #[test]
    fn test_shift_be() {
        let mut bytes1 = [
            0b00011111,
            0b11111111,
            0b11111000,
        ];
        let mut bytes2 = [
            0b11111111,
            0b11111111,
            0b11111111,
        ];
        let target = [
            0b00000011,
            0b11111111,
            0b11111111,
        ];

        BigEndian::shift_bytes::<U4, U3>(&mut bytes1);
        BigEndian::shift_bytes::<U4, U3>(&mut bytes2);

        assert_eq!(bytes1, target);
        assert_eq!(bytes2, target);

        let mut bytes1 = [
            0b00011111,
            0b11101111,
            0b11111000,
        ];
        let mut bytes2 = [
            0b11111111,
            0b11101111,
            0b11111111,
        ];
        let target = [
            0b00000011,
            0b11111101,
            0b11111111,
        ];

        BigEndian::shift_bytes::<U4, U3>(&mut bytes1);
        BigEndian::shift_bytes::<U4, U3>(&mut bytes2);

        //panic!("\n0b{:08b}\n0b{:08b}\n0b{:08b}", bytes1[0], bytes1[1], bytes1[2]);
        assert_eq!(bytes1, target);
        assert_eq!(bytes2, target);
    }
}