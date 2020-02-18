#![cfg_attr(not(test), no_std)]

use packing::*;
use core::{
    mem::size_of,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Spaff {
    Spooge = 0x1,
    Spoodge = 0x2,
}

impl Packed<U7, U0, U1> for Spaff {
    type Error = Error;

    const BYTES: usize = size_of::<Self>();
    fn unpack<En: Endian>(bytes: &[u8]) -> Result<Self, Self::Error> {
        assert!(bytes.len() == size_of::<Self>());
        match bytes[0] {
            0x1 => Ok(Spaff::Spooge),
            0x2 => Ok(Spaff::Spoodge),
            _ => Err(Error::InvalidEnumDiscriminant),
        }
    }

    fn pack<En: Endian>(&self, bytes: &mut [u8]) -> Result<(), Self::Error> { 
        assert!(bytes.len() == size_of::<Self>());
        bytes[0] = match self {
            Spaff::Spooge => 0x1,
            Spaff::Spoodge => 0x2,
        };
        Ok(())
    } 
}

#[derive(Packed)]
#[packed(big_endian, lsb0)]
pub struct ModeSense6Command {
    #[packed(start_bit=7, end_bit=0, start_byte=0, end_byte=0)] pub op_code: Spaff,
    #[packed(start_bit=3, end_bit=3, start_byte=1, end_byte=1)] pub disable_block_descriptors: bool,
    #[packed(start_bit=7, end_bit=6, start_byte=2, end_byte=2)] pub page_control: u8,
    #[packed(start_bit=5, end_bit=0, start_byte=2, end_byte=2)] pub page_code: u8,
    #[packed(start_bit=7, end_bit=0, start_byte=3, end_byte=3)] pub subpage_code: u8,
    #[packed(start_bit=7, end_bit=0, start_byte=4, end_byte=4)] pub allocation_length: u8,
    #[packed(start_bit=7, end_bit=0, start_byte=5, end_byte=5)] pub control: u8,
    #[packed(start_bit=7, end_bit=0, start_byte=6, end_byte=7)] pub sixteen: u16,
}

#[test]
fn test_mode_sense_6_unpack() {
    let op_code = 0x01;
    let disable_block_descriptors = true;
    let page_control = 3;
    let page_code = 4;
    let subpage_code = 222;
    let allocation_length = 1;
    let control = 41;
    let sixteen = u8::max_value() as u16 + 5;

    let sixteen_bytes = sixteen.to_be_bytes();

    let bytes = [
        op_code,
        (disable_block_descriptors as u8) << 3,
        page_control << 6 | page_code,
        subpage_code,
        allocation_length,
        control,
        sixteen_bytes[0],
        sixteen_bytes[1],
    ];

    let cmd = ModeSense6Command::unpack::<LittleEndian>(&bytes).unwrap();
    //assert_eq!(op_code, cmd.op_code);
    assert_eq!(disable_block_descriptors, cmd.disable_block_descriptors);
    assert_eq!(page_control, cmd.page_control);
    assert_eq!(page_code, cmd.page_code);
    assert_eq!(subpage_code, cmd.subpage_code);
    assert_eq!(allocation_length, cmd.allocation_length);
    assert_eq!(control, cmd.control);

    let mut packed = [0; ModeSense6Command::PACK_BYTES_LEN];
    cmd.pack::<LittleEndian>(&mut packed).unwrap();
    assert_eq!(bytes, packed);
}

#[cfg(test)]
mod tests {
    use packing::*;
    fn test_bits<S: Bit, E: Bit>(width: usize) {
        assert!(width <= 16);
        assert!(width > 0);

        if width == 1 && S::USIZE < E::USIZE {
            return;
        }
        
        let mut b = Vec::new();
        for _ in 0..width {
            b.push(0xFF);
        }

        let bytes: &mut [u8] = &mut b[..];
        let total_bits = match width {
            1 => S::U32 +1 - E::U32,
            _ => S::U32 + 1 + (8 - E::U32) + 8 * (width.max(2) as u32 - 2),
        };

        let expected = if total_bits < (16 * 8) {
            (2_u128.pow(total_bits) - 1) as u128
        } else {
            u128::max_value()
        };

        match width {
            1 => {
                assert_eq!(expected as u16, <u16 as Packed<S, E, U1>>::unpack::<LittleEndian>(bytes).unwrap());
                assert_eq!(expected as u32, <u32 as Packed<S, E, U1>>::unpack::<LittleEndian>(bytes).unwrap());
                assert_eq!(expected as u64, <u64 as Packed<S, E, U1>>::unpack::<LittleEndian>(bytes).unwrap());
                assert_eq!(expected as u128, <u128 as Packed<S, E, U1>>::unpack::<LittleEndian>(bytes).unwrap());
            },
            2 => {
                assert_eq!(expected as u16, <u16 as Packed<S, E, U2>>::unpack::<LittleEndian>(bytes).unwrap());
                assert_eq!(expected as u32, <u32 as Packed<S, E, U2>>::unpack::<LittleEndian>(bytes).unwrap());
                assert_eq!(expected as u64, <u64 as Packed<S, E, U2>>::unpack::<LittleEndian>(bytes).unwrap());
                assert_eq!(expected as u128, <u128 as Packed<S, E, U2>>::unpack::<LittleEndian>(bytes).unwrap());
            },
            3 => {
                assert_eq!(expected as u32, <u32 as Packed<S, E, U3>>::unpack::<LittleEndian>(bytes).unwrap());
                assert_eq!(expected as u64, <u64 as Packed<S, E, U3>>::unpack::<LittleEndian>(bytes).unwrap());
                assert_eq!(expected as u128, <u128 as Packed<S, E, U3>>::unpack::<LittleEndian>(bytes).unwrap());
            },
            4 => {
                assert_eq!(expected as u32, <u32 as Packed<S, E, U4>>::unpack::<LittleEndian>(bytes).unwrap());
                assert_eq!(expected as u64, <u64 as Packed<S, E, U4>>::unpack::<LittleEndian>(bytes).unwrap());
                assert_eq!(expected as u128, <u128 as Packed<S, E, U4>>::unpack::<LittleEndian>(bytes).unwrap());
            },
            5 => {
                assert_eq!(expected as u64, <u64 as Packed<S, E, U5>>::unpack::<LittleEndian>(bytes).unwrap());
                assert_eq!(expected as u128, <u128 as Packed<S, E, U5>>::unpack::<LittleEndian>(bytes).unwrap());
            },
            6 => {
                assert_eq!(expected as u64, <u64 as Packed<S, E, U6>>::unpack::<LittleEndian>(bytes).unwrap());
                assert_eq!(expected as u128, <u128 as Packed<S, E, U6>>::unpack::<LittleEndian>(bytes).unwrap());
            },
            7 => {
                assert_eq!(expected as u64, <u64 as Packed<S, E, U7>>::unpack::<LittleEndian>(bytes).unwrap());
                assert_eq!(expected as u128, <u128 as Packed<S, E, U7>>::unpack::<LittleEndian>(bytes).unwrap());
            },
            8 => {
                assert_eq!(expected as u64, <u64 as Packed<S, E, U8>>::unpack::<LittleEndian>(bytes).unwrap());
                assert_eq!(expected as u128, <u128 as Packed<S, E, U8>>::unpack::<LittleEndian>(bytes).unwrap());
            },
            15 => assert_eq!(expected as u128, <u128 as Packed<S, E, U15>>::unpack::<LittleEndian>(bytes).unwrap()),
            16 => assert_eq!(expected as u128, <u128 as Packed<S, E, U16>>::unpack::<LittleEndian>(bytes).unwrap()),
            _ => {},
        }
        
    }

    fn test_all_bits(width: usize) {
        test_bits::<U0, U0>(width);    test_bits::<U1, U0>(width);    test_bits::<U2, U0>(width);
        test_bits::<U3, U0>(width);    test_bits::<U4, U0>(width);    test_bits::<U5, U0>(width);
        test_bits::<U6, U0>(width);    test_bits::<U7, U0>(width);

        test_bits::<U0, U1>(width);    test_bits::<U1, U1>(width);    test_bits::<U2, U1>(width);
        test_bits::<U3, U1>(width);    test_bits::<U4, U1>(width);    test_bits::<U5, U1>(width);
        test_bits::<U6, U1>(width);    test_bits::<U7, U1>(width);

        test_bits::<U0, U2>(width);    test_bits::<U1, U2>(width);    test_bits::<U2, U2>(width);
        test_bits::<U3, U2>(width);    test_bits::<U4, U2>(width);    test_bits::<U5, U2>(width);
        test_bits::<U6, U2>(width);    test_bits::<U7, U2>(width);

        test_bits::<U0, U3>(width);    test_bits::<U1, U3>(width);    test_bits::<U2, U3>(width);
        test_bits::<U3, U3>(width);    test_bits::<U4, U3>(width);    test_bits::<U5, U3>(width);
        test_bits::<U6, U3>(width);    test_bits::<U7, U3>(width);

        test_bits::<U0, U4>(width);    test_bits::<U1, U4>(width);    test_bits::<U2, U4>(width);
        test_bits::<U3, U4>(width);    test_bits::<U4, U4>(width);    test_bits::<U5, U4>(width);
        test_bits::<U6, U4>(width);    test_bits::<U7, U4>(width);

        test_bits::<U0, U5>(width);    test_bits::<U1, U5>(width);    test_bits::<U2, U5>(width);
        test_bits::<U3, U5>(width);    test_bits::<U4, U5>(width);    test_bits::<U5, U5>(width);
        test_bits::<U6, U5>(width);    test_bits::<U7, U5>(width);

        test_bits::<U0, U6>(width);    test_bits::<U1, U6>(width);    test_bits::<U2, U6>(width);
        test_bits::<U3, U6>(width);    test_bits::<U4, U6>(width);    test_bits::<U5, U6>(width);
        test_bits::<U6, U6>(width);    test_bits::<U7, U6>(width);

        test_bits::<U0, U7>(width);    test_bits::<U1, U7>(width);    test_bits::<U2, U7>(width);
        test_bits::<U3, U7>(width);    test_bits::<U4, U7>(width);    test_bits::<U5, U7>(width);
        test_bits::<U6, U7>(width);    test_bits::<U7, U7>(width);
    }

    #[test]
    fn test_le() {
        test_all_bits(1);
        test_all_bits(2);
        test_all_bits(4);
        test_all_bits(8);
        test_all_bits(15);
        test_all_bits(16);
    }
}