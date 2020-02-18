use crate::{
    Bit,
    Unsigned, U1, IsGreaterOrEqual,
};

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
    const IS_LITTLE: bool;
    /// Align the bits in slice to usual 8 bit byte boundaries in the endianness represented by 
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
    fn align_bytes<S: Bit, E: Bit>(bytes: &mut [u8]);


    /// Take nice 8 bit aligned bytes and shift them to align with the bits specified by `S` and `E`
    /// in an endian aware way - that means the most significant bits will be masked away rather than
    /// the least significant bits
    ///
    /// Also masks away bytes outside the range specified by `S` and `E`.
    /// Does nothing if `S` == 7 and `E` == 0.
    ///
    /// `S` and `E` type parameters represent bit positions with 7 being the most significant bit and
    /// 0 being the least significant bit. `S` is the first included bit in the first byte of the slice.
    /// `E` is the last included bit in the last byte of the slice.
    ///
    /// Panics if `bytes`.len() == 0 or if `bytes`.len() == 1 AND S < E 
    fn unalign_bytes<S: Bit, E: Bit>(bytes: &mut [u8]);

    /// Copies `W` bytes from `src` to `dest` in an endian aware way. Uses `copy_from_slice` (memcpy)
    ///
    /// LittleEndian will copy the bytes to the front, BigEndian will copy the bytes to the back. 
    ///
    /// Panics if `src`.len() != `W::USIZE` or `dest`.len() < `W::USIZE`
    fn copy_bytes<W: Unsigned>(src: &[u8], dest: &mut [u8])
    where
        W: IsGreaterOrEqual<U1>;

    fn merge_field<S: Bit, E: Bit, W: Unsigned>(src: &[u8], dest: &mut [u8])
    where 
        W: IsGreaterOrEqual<U1>;
}

/// Defines little endian bit shifting required for packing and unpacking non aligned fields in byte slices
///
/// Not construcatable, used only at type level
pub enum LittleEndian {}
impl Endian for LittleEndian {
    const IS_LITTLE: bool = true;
    /// Align the bits in slice to usual 8 bit byte boundaries - specifcally for `LittleEndian` this means
    /// shifting the bits towards the top of the slice, copying the LSB from the next byte if there is
    /// space in the previous byte. Any space between the first bit and `S` and between `E` and the last bit
    /// ends up in the MSB of the last byte
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
    fn align_bytes<S: Bit, E: Bit>(bytes: &mut [u8]) {
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

    /// Take nice 8 bit aligned bytes and shift them to align with the bits specified by `S` and `E`
    ///
    /// Specifcally for `LittleEndian` this means shifting the bits right and down to make `S` bits space
    /// at the beginning then masking off the MSB of the last byte to make `E` bits space at the end
    ///
    /// `S` and `E` type parameters represent bit positions with 7 being the most significant bit and
    /// 0 being the least significant bit. `S` is the first included bit in the first byte of the slice.
    /// `E` is the last included bit in the last byte of the slice.
    ///
    /// Panics if `bytes`.len() == 0 or if `bytes`.len() == 1 AND S < E 
    fn unalign_bytes<S: Bit, E: Bit>(bytes: &mut [u8]) {
        let len = bytes.len();
        
        // Not valid to call this with no data
        assert!(len > 0);
        // Not valid to call with 1 byte and S E bits overlapping
        assert!(len > 1 || S::USIZE >= E::USIZE);

        // If start and end are both at their respective byte boundaries there's nothing to do here
        if S::USIZE == 7 && E::USIZE == 0 { return; }

        // If the first bit is shifted from the start
        if S::USIZE != 7 {
            // Shift all the MSBs into the next byte
            for i in (1..len).rev() {
                let j = i - 1;

                // Make space for the incoming bits. This may discard MSB bits if number of bits
                // in the source is great than will fit in the new field specified by S and E
                // Range checking around this is done in the Packed::pack impelementations
                bytes[i] <<= 7-S::USIZE;

                // Add the bits we'll lose in the next iteration into this byte
                bytes[i] |= bytes[j] >> (S::USIZE + 1);
            }

            //Mask away anything left before the S position
            bytes[0] &= S::HEAD_MASK;
        }

        // Shift the final bits from 0 to the new E bit. This discards the MSB if the total
        // number of bits in the source is > the total amount of space insidcated by S and E.
        // It also makes everything < E zero
        bytes[len-1] <<= E::USIZE;
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

    fn merge_field<S: Bit, E: Bit, W: Unsigned>(src: &[u8], dest: &mut [u8])
    where 
        W: IsGreaterOrEqual<U1> 
    {
        assert!(src.len() == W::USIZE);
        assert!(dest.len() >= W::USIZE);

        // Not valid to call with 1 byte and S E bits overlapping
        assert!(W::USIZE > 1 || S::USIZE >= E::USIZE);

        if S::USIZE == 7 && E::USIZE == 0 {
            LittleEndian::copy_bytes::<W>(src, dest);
        } else if W::USIZE == 1 {
            dest[0] &= S::TAIL_MASK | E::HEAD_MASK;
            dest[0] |= src[0];
        } else {
            dest[0] &= S::TAIL_MASK;
            dest[0] |= src[0];

            dest[W::USIZE - 1] &= E::HEAD_MASK;
            dest[W::USIZE - 1] |= src[W::USIZE - 1];

            if W::USIZE > 2 {
                dest[1..(W::USIZE - 1)].copy_from_slice(&src[1..(W::USIZE - 1)]);
            }
        }
    }
}

/// Defines big endian bit shifting required for packing and unpacking non aligned fields in byte slices
///
/// Not construcatable, used only at type level
pub enum BigEndian {}
impl Endian for BigEndian {
    const IS_LITTLE: bool = false;
    /// Align the bits in slice to usual 8 bit byte boundaries - specifcally for `BigEndian` this means
    /// shifting the bits towards the bottom of the slice, copying the MSB from the previous byte if there is
    /// space in the next byte. Any space between the first bit and `S` and between `E` and the last bit
    /// ends up in the MSB of the first byte
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
    fn align_bytes<S: Bit, E: Bit>(bytes: &mut [u8]) {
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

    /// Take nice 8 bit aligned bytes and shift them to align with the bits specified by `S` and `E`
    ///
    /// Specifcally for `BigEndian` this means shifting the bits left and up to make `E` bits space
    /// at the end then masking off the MSB of the first byte to make `S` bits space at the beginning
    ///
    /// `S` and `E` type parameters represent bit positions with 7 being the most significant bit and
    /// 0 being the least significant bit. `S` is the first included bit in the first byte of the slice.
    /// `E` is the last included bit in the last byte of the slice.
    ///
    /// Panics if `bytes`.len() == 0 or if `bytes`.len() == 1 AND S < E 
    fn unalign_bytes<S: Bit, E: Bit>(bytes: &mut [u8]) {
        let len = bytes.len();
        
        // Not valid to call this with no data
        assert!(len > 0);
        // Not valid to call with 1 byte and S E bits overlapping
        assert!(len > 1 || S::USIZE >= E::USIZE);

        // If start and end are both at their respective byte boundaries there's nothing to do here
        if S::USIZE == 7 && E::USIZE == 0 { return; }

        // If the last bit is shifted from the end
        if E::USIZE != 0 {
            // Shift all the MSBs into the previous byte
            for i in 0..(len-1) {
                let j = i + 1;

                // Make space for the incoming bits. This may discard MSB bits if number of bits
                // in the source is great than will fit in the new field specified by S and E
                // Range checking around this is done in the Packed::pack impelementations
                bytes[i] <<= E::USIZE;

                // Add the bits we'll lose in the next iteration into this byte
                bytes[i] |= bytes[j] >> (7-E::USIZE);
            }

            // Shift the final bits from 0 to the new E bit. This discards the MSB but we should just
            // added them to the previous byte in the above loop. It also makes everything < E zero
            bytes[len-1] <<= E::USIZE;

        }

        //Mask away anything left before the S position
        bytes[0] &= S::HEAD_MASK;
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

    fn merge_field<S: Bit, E: Bit, W: Unsigned>(src: &[u8], dest: &mut [u8])
    where 
        W: IsGreaterOrEqual<U1> 
    {
        assert!(src.len() == W::USIZE);
        assert!(dest.len() >= W::USIZE);

        // Not valid to call with 1 byte and S E bits overlapping
        assert!(W::USIZE > 1 || S::USIZE >= E::USIZE);

        if S::USIZE == 7 && E::USIZE == 0 {
            BigEndian::copy_bytes::<W>(src, dest);
        } else if W::USIZE == 1 {
            let s = dest.len() - W::USIZE;
            dest[s] &= S::TAIL_MASK | E::HEAD_MASK;
            dest[s] |= src[0];
        } else {
            let s = dest.len() - W::USIZE;
            dest[s] &= S::TAIL_MASK;
            dest[s] |= src[0];

            dest[s + W::USIZE - 1] &= E::HEAD_MASK;
            dest[s + W::USIZE - 1] |= src[W::USIZE - 1];

            if W::USIZE > 2 {
                dest[(s+1)..(s + W::USIZE - 1)].copy_from_slice(&src[1..(W::USIZE - 1)]);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_le_merge() {
        let mut dst = [0];
        let mut src = [0];
        let mut target = 0;

        dst[0] = 0b00000000;
        src[0] = 0b01111110;
        target = 0b01111110;
        LittleEndian::merge_field::<U6, U1, U1>(&src, &mut dst);
        assert_eq!(target, dst[0]);

        dst[0] = 0b10001000;
        src[0] = 0b00100100;
        target = 0b10100100;
        LittleEndian::merge_field::<U6, U1, U1>(&src, &mut dst);
        assert_eq!(target, dst[0]);

        let mut dst = [0, 0, 0, 0];
        let mut src = [0, 0, 0, 0];
        let mut tgt = [0, 0, 0, 0];

        dst[0] = 0b10001000;
        src[0] = 0b00100100;
        tgt[0] = 0b10100100;

        dst[1] = 0b01010101;
        src[1] = 0b11111111;
        tgt[1] = 0b11111111;

        dst[2] = 0b11111111;
        src[2] = 0b00000000;
        tgt[2] = 0b00000000;

        dst[3] = 0b11111111;
        src[3] = 0b00000000;
        tgt[3] = 0b00000011;

        LittleEndian::merge_field::<U6, U1, U4>(&src, &mut dst);
        assert_eq!(tgt, dst);

        let mut dst = [0, 0, 0, 0];
        let mut src = [0, 0];
        let mut tgt = [0, 0, 0, 0];

        dst[0] = 0b10001000;
        tgt[0] = 0b10100100;
        src[0] = 0b00100100;

        dst[1] = 0b01010101;
        src[1] = 0b11111100;
        tgt[1] = 0b11111101;

        dst[2] = 0b01010101;
        tgt[2] = dst[2];

        dst[3] = 0b10101010;
        tgt[3] = dst[3];

        LittleEndian::merge_field::<U6, U1, U2>(&src, &mut dst);
        if tgt != dst {
            panic!("\n0b{:08b}   0b{:08b}   0b{:08b}\n0b{:08b}   0b{:08b}   0b{:08b}\n0b{:08b}   0b{:08b}\n0b{:08b}   0b{:08b}\n", 
                dst[0], src[0], tgt[0],
                dst[1], src[1], tgt[1],
                dst[2], tgt[2],
                dst[3], tgt[3],
            );
        }
        assert_eq!(tgt, dst);
    }

    #[test]
    fn test_be_merge() {
        let mut dst = [0];
        let mut src = [0];
        let mut target = 0;

        dst[0] = 0b00000000;
        src[0] = 0b01111110;
        target = 0b01111110;
        BigEndian::merge_field::<U6, U1, U1>(&src, &mut dst);
        assert_eq!(target, dst[0]);

        dst[0] = 0b10001000;
        src[0] = 0b00100100;
        target = 0b10100100;
        BigEndian::merge_field::<U6, U1, U1>(&src, &mut dst);
        assert_eq!(target, dst[0]);

        let mut dst = [0, 0, 0, 0];
        let mut src = [0, 0, 0, 0];
        let mut tgt = [0, 0, 0, 0];

        dst[0] = 0b10001000;
        src[0] = 0b00100100;
        tgt[0] = 0b10100100;

        dst[1] = 0b01010101;
        src[1] = 0b11111111;
        tgt[1] = 0b11111111;

        dst[2] = 0b11111111;
        src[2] = 0b00000000;
        tgt[2] = 0b00000000;

        dst[3] = 0b11111111;
        src[3] = 0b00000000;
        tgt[3] = 0b00000011;

        BigEndian::merge_field::<U6, U1, U4>(&src, &mut dst);
        if tgt != dst {
            panic!("\n0b{:08b}   0b{:08b}   0b{:08b}\n0b{:08b}   0b{:08b}   0b{:08b}\n0b{:08b}   0b{:08b}   0b{:08b}\n0b{:08b}   0b{:08b}   0b{:08b}\n", 
                dst[0], src[0], tgt[0],
                dst[1], src[1], tgt[1],
                dst[2], src[2], tgt[2],
                dst[3], src[3], tgt[3],
            );
        }
        assert_eq!(tgt, dst);

        let mut dst = [0, 0, 0, 0];
        let mut src = [0, 0];
        let mut tgt = [0, 0, 0, 0];

        dst[2] = 0b10001000;
        tgt[2] = 0b10100100;
        src[0] = 0b00100100;

        dst[3] = 0b01010101;
        tgt[3] = 0b11111101;
        src[1] = 0b11111100;

        dst[0] = 0b01010101;
        tgt[0] = dst[0];

        dst[1] = 0b10101010;
        tgt[1] = dst[1];

        BigEndian::merge_field::<U6, U1, U2>(&src, &mut dst);
        if tgt != dst {
            panic!("\n0b{:08b}                0b{:08b} {}\n0b{:08b}                0b{:08b} {}\n0b{:08b}   0b{:08b}   0b{:08b} {}\n0b{:08b}   0b{:08b}   0b{:08b} {}\n", 
                dst[0], tgt[0], dst[0] == tgt[0],
                dst[1], tgt[1], dst[1] == tgt[1],
                dst[2], src[0], tgt[2], dst[2] == tgt[2],
                dst[3], src[1], tgt[3], dst[3] == tgt[3],
            );
        }
        assert_eq!(tgt, dst);
    }

    #[test]
    fn test_le() {
        let bytes1 = [
            0b00011111,
            0b11111111,
            0b11111000,
        ];
        let bytes2 = [
            0b11111111,
            0b11111111,
            0b11111111,
        ];
        let target = [
            0b11111111,
            0b11111111,
            0b00000011,
        ];

        let mut b1 = bytes1.clone();
        let mut b2 = bytes2.clone();

        LittleEndian::align_bytes::<U4, U3>(&mut b1);
        assert_eq!(b1, target);

        LittleEndian::align_bytes::<U4, U3>(&mut b2);
        assert_eq!(b2, target);

        LittleEndian::unalign_bytes::<U4, U3>(&mut b1);
        assert_eq!(b1, bytes1);

        LittleEndian::unalign_bytes::<U4, U3>(&mut b2);
        // Can't just compare the bytes because the 1s outside of S and E were lost
        for i in 0..b1.len() {
            assert!(bytes1[i] & b1[i] == b1[i]);
        }

        let bytes1 = [
            0b00011111,
            0b11101111,
            0b11111000,
        ];
        let bytes2 = [
            0b11111111,
            0b11101111,
            0b11111111,
        ];
        let target = [
            0b11111111,
            0b11111101,
            0b00000011,
        ];

        let mut b1 = bytes1.clone();
        let mut b2 = bytes2.clone();

        LittleEndian::align_bytes::<U4, U3>(&mut b1);
        assert_eq!(b1, target);

        LittleEndian::align_bytes::<U4, U3>(&mut b2);
        assert_eq!(b2, target);

        LittleEndian::unalign_bytes::<U4, U3>(&mut b1);
        /*panic!("\n0b{:08b}   0b{:08b}\n0b{:08b}   0b{:08b}\n0b{:08b}   0b{:08b}", 
            bytes1[0], b1[0],
            bytes1[1], b1[1],
            bytes1[2], b1[2],
        );
        */
        assert_eq!(b1, bytes1);

        LittleEndian::unalign_bytes::<U4, U3>(&mut b2);
        // Can't just compare the bytes because the 1s outside of S and E were lost
        for i in 0..b1.len() {
            assert!(bytes1[i] & b1[i] == b1[i]);
        }
    }

    #[test]
    fn test_shift_be() {
        let bytes1 = [
            0b00011111,
            0b11111111,
            0b11111000,
        ];
        let bytes2 = [
            0b11111111,
            0b11111111,
            0b11111111,
        ];
        let target = [
            0b00000011,
            0b11111111,
            0b11111111,
        ];

        let mut b1 = bytes1.clone();
        let mut b2 = bytes2.clone();

        BigEndian::align_bytes::<U4, U3>(&mut b1);
        assert_eq!(b1, target);

        BigEndian::align_bytes::<U4, U3>(&mut b2);
        assert_eq!(b2, target);

        BigEndian::unalign_bytes::<U4, U3>(&mut b1);
        /*panic!("\n0b{:08b}   0b{:08b}\n0b{:08b}   0b{:08b}\n0b{:08b}   0b{:08b}", 
            bytes1[0], b1[0],
            bytes1[1], b1[1],
            bytes1[2], b1[2],
        );
        */
        assert_eq!(b1, bytes1);        

        BigEndian::unalign_bytes::<U4, U3>(&mut b2);
        // Can't just compare the bytes because the 1s outside of S and E were lost
        for i in 0..b1.len() {
            assert!(bytes1[i] & b1[i] == b1[i]);
        }

        let bytes1 = [
            0b00011111,
            0b11101111,
            0b11111000,
        ];
        let bytes2 = [
            0b11111111,
            0b11101111,
            0b11111111,
        ];
        let target = [
            0b00000011,
            0b11111101,
            0b11111111,
        ];

        let mut b1 = bytes1.clone();
        let mut b2 = bytes2.clone();

        BigEndian::align_bytes::<U4, U3>(&mut b1);
        assert_eq!(b1, target);

        BigEndian::align_bytes::<U4, U3>(&mut b2);
        assert_eq!(b2, target);

        BigEndian::unalign_bytes::<U4, U3>(&mut b1);
        /*
        panic!("\n0b{:08b}   0b{:08b}\n0b{:08b}   0b{:08b}\n0b{:08b}   0b{:08b}", 
            bytes1[0], b1[0],
            bytes1[1], b1[1],
            bytes1[2], b1[2],
        );
        */
        assert_eq!(b1, bytes1);        

        BigEndian::unalign_bytes::<U4, U3>(&mut b2);
        // Can't just compare the bytes because the 1s outside of S and E were lost
        for i in 0..b1.len() {
            assert!(bytes1[i] & b1[i] == b1[i]);
        }
    }

    #[test]
    fn test_pack() {
        struct Moo {
            op_code: u8,
        };

        let moo = Moo { op_code: 10 };
        let mut bytes = [0; 99];
        //<moo.op_code as Packed<U7, U0, U1>>::pack::<packing::BigEndian>(&mut bytes [0usize ..= 0usize])?
        //<u8 as Packed<&[u8], U7, U0, U1>>::pack::<BigEndian>(&moo.op_code, &bytes[0..=0]);
    }
}