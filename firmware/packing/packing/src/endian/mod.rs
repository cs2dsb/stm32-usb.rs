use crate::Bit;

mod little;
pub use little::*;

mod big;
pub use big::*;

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
    /// Simple memcopy if `S` == 7 and `E` == 0.
    ///
    /// `S` and `E` type parameters represent bit positions with 7 being the most significant bit and
    /// 0 being the least significant bit. `S` is the first included bit in the first byte of the slice.
    /// `E` is the last included bit in the last byte of the slice.
    fn align_field_bits<S: Bit, E: Bit>(input_bytes: &[u8], output_bytes: &mut [u8]);

    /// Take nice 8 bit aligned bytes and shift them to align with the bits specified by `S` and `E`
    /// in an endian aware way - this means the most significant bits will be masked away rather than
    /// the least significant bits. Does not perform any range checks to determine if data is being 
    /// truncated.
    ///
    /// Data is ORed into the output byte slice so it is acceptable for there to be data already in
    /// the slice outside of the field defined by `S` and `E`. Bits within the field must be set to 0
    /// prior to calling this function.
    /// TODO: Clear out data inside the field so dirty buffers can be reused. 
    ///
    /// `S` and `E` type parameters represent bit positions with 7 being the most significant bit and
    /// 0 being the least significant bit. `S` is the first included bit in the first byte of the slice.
    /// `E` is the last included bit in the last byte of the slice.
    fn restore_field_bits<S: Bit, E: Bit>(input_bytes: &[u8], output_bytes: &mut [u8]);
}




#[cfg(test)]
mod tests {
    use crate::*;
    use core::mem::size_of;
    extern crate test;
    use test::{ black_box, Bencher };

    /*
        Cases
        1. aligned - just copy
            1. Correct length - just copy
            2. Too short - shift and copy
            3. Too long - not supported
        2. not aligned
            1. Within normal bounds - shift and copy
            2. Straddling - shift, shrink, copy
            3. 

        let mut new_bytes = [0; size_of::<Self>];
        if sb == 7 && eb == 0 {

        }

    */

    #[bench]
    fn new_per_field(b: &mut Bencher) {
        let mut num: u32 = black_box(42);

        b.iter(|| {
            num = black_box(num + 1);
            let bytes = num.to_le_bytes();
            {
                let mut bytes = [0; size_of::<u32>() + 1];
                bytes[1..].copy_from_slice(&bytes);
                let mut new_num = [0; size_of::<u32>()];
                for i in 0..(bytes.len() - 1) {
                    new_num[i] = black_box(bytes[i + 1]);
                }
                let new_num = black_box(u32::from_le_bytes(new_num));
                assert_eq!(new_num, num);
            }
            num = black_box(num + 1);
            let bytes = num.to_le_bytes();
            {
                let mut bytes = [0; size_of::<u32>() + 1];
                bytes[1..].copy_from_slice(&bytes);
                let mut new_num = [0; size_of::<u32>()];
                for i in 0..(bytes.len() - 1) {
                    new_num[i] = black_box(bytes[i + 1]);
                }
                let new_num = black_box(u32::from_le_bytes(new_num));
                assert_eq!(new_num, num);
            }
            num = black_box(num + 1);
            let bytes = num.to_le_bytes();
            {
                let mut bytes = [0; size_of::<u32>() + 1];
                bytes[1..].copy_from_slice(&bytes);
                let mut new_num = [0; size_of::<u32>()];
                for i in 0..(bytes.len() - 1) {
                    new_num[i] = black_box(bytes[i + 1]);
                }
                let new_num = black_box(u32::from_le_bytes(new_num));
                assert_eq!(new_num, num);
            }
            num = black_box(num + 1);
            let bytes = num.to_le_bytes();
            {
                let mut bytes = [0; size_of::<u32>() + 1];
                bytes[1..].copy_from_slice(&bytes);
                let mut new_num = [0; size_of::<u32>()];
                for i in 0..(bytes.len() - 1) {
                    new_num[i] = black_box(bytes[i + 1]);
                }
                let new_num = black_box(u32::from_le_bytes(new_num));
                assert_eq!(new_num, num);
            }
            num = black_box(num + 1);
            let bytes = num.to_le_bytes();
            {
                let mut bytes = [0; size_of::<u32>() + 1];
                bytes[1..].copy_from_slice(&bytes);
                let mut new_num = [0; size_of::<u32>()];
                for i in 0..(bytes.len() - 1) {
                    new_num[i] = black_box(bytes[i + 1]);
                }
                let new_num = black_box(u32::from_le_bytes(new_num));
                assert_eq!(new_num, num);
            }
        })
    }

    #[bench]
    fn new_per_struct(b: &mut Bencher) {
        let mut num: u32 = black_box(42);

        b.iter(|| {
            let mut bytes = [0; size_of::<u32>() + 1];
            num = black_box(num + 1);
            let bytes = num.to_le_bytes();
            {
                bytes[1..].copy_from_slice(&bytes);
                let mut new_num = [0; size_of::<u32>()];
                for i in 0..(bytes.len() - 1) {
                    new_num[i] = black_box(bytes[i + 1]);
                }
                let new_num = black_box(u32::from_le_bytes(new_num));
                assert_eq!(new_num, num);
            }
            num = black_box(num + 1);
            let bytes = num.to_le_bytes();
            {
                bytes[1..].copy_from_slice(&bytes);
                let mut new_num = [0; size_of::<u32>()];
                for i in 0..(bytes.len() - 1) {
                    new_num[i] = black_box(bytes[i + 1]);
                }
                let new_num = black_box(u32::from_le_bytes(new_num));
                assert_eq!(new_num, num);
            }
            num = black_box(num + 1);
            let bytes = num.to_le_bytes();
            {
                bytes[1..].copy_from_slice(&bytes);
                let mut new_num = [0; size_of::<u32>()];
                for i in 0..(bytes.len() - 1) {
                    new_num[i] = black_box(bytes[i + 1]);
                }
                let new_num = black_box(u32::from_le_bytes(new_num));
                assert_eq!(new_num, num);
            }
            num = black_box(num + 1);
            let bytes = num.to_le_bytes();
            {
                bytes[1..].copy_from_slice(&bytes);
                let mut new_num = [0; size_of::<u32>()];
                for i in 0..(bytes.len() - 1) {
                    new_num[i] = black_box(bytes[i + 1]);
                }
                let new_num = black_box(u32::from_le_bytes(new_num));
                assert_eq!(new_num, num);
            }
            num = black_box(num + 1);
            let bytes = num.to_le_bytes();
            {
                bytes[1..].copy_from_slice(&bytes);
                let mut new_num = [0; size_of::<u32>()];
                for i in 0..(bytes.len() - 1) {
                    new_num[i] = black_box(bytes[i + 1]);
                }
                let new_num = black_box(u32::from_le_bytes(new_num));
                assert_eq!(new_num, num);
            }
        })
    }

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