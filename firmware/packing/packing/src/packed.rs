use crate::{
    Unsigned,
    Bit,
    Endian,
};

/// Trait that provides packing and unpacking of arbitrarily aligned endian aware fields
/// to/from byte slices
pub trait Packed<S: Bit, E: Bit, W: Unsigned> where Self: Sized {
    type Error;
    /// Number of bytes implementation packs/unpacks to/from
    ///
    /// Note `W` type parameter influences the size for a given implementation. For example, a
    /// `u32` can be packed or unpacked from 1, 2, 3 or 4 bytes
    const BYTES: usize;

    /// Unpacks provided bytes into `Self` in an endian aware way
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
    fn unpack<En: Endian>(bytes: &[u8]) -> Result<Self, Self::Error>;

    /// Packs self into provided mutable reference to a byte slice in an endian aware way
    ///
    /// In general, implementations use to_le_bytes or to_be_bytes to generate the correct byte 
    /// represenations from primitive types which will allocate small temporary arrays on the stack.
    /// Benchmarking showed that using to_x_bytes was as fast or faster in most cases than hand written
    /// bit shifting. Some shifting is still required if S or E indicate a non byte aligned field.
    fn pack<En: Endian>(&self, bytes: &mut [u8]) -> Result<(), Self::Error>; 

    fn update_from_packed<En: Endian>(&mut self, bytes: &[u8]) -> Result<(), Self::Error> {
        *self = Self::unpack::<En>(bytes)?;
        Ok(())
    }
}