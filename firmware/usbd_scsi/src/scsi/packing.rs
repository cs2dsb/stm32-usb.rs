use packed_struct::PackedStruct;
use super::Error;

pub trait Len {
    fn len() -> usize;
}

macro_rules! define_len {
    ($($len: literal$(,)?)+) => {
        $(
            impl Len for [u8; $len] {
                fn len() -> usize { $len }
            }
        )+
    }
}

define_len!(512, 255, 31, 15, 11, 9, 5);

pub trait ResizeSmaller<B: Len>: Len {
    fn resize(&self) -> &B {
        assert!(Self::len() >= B::len());
        unsafe { &*(self as *const _ as *const _) }
    }
}

macro_rules! define_resize_smaller {
    ($from: literal, $($to: literal$(,)?)+ ) => {
        $(
            impl ResizeSmaller<[u8; $to]> for [u8; $from] {}
        )+
    }
}

define_resize_smaller!(512, 31, 15, 11, 9, 5);
define_resize_smaller!(255, 31, 15, 11, 9, 5);
define_resize_smaller!(15, 11, 9, 5);

pub trait ParsePackedStruct<A: ResizeSmaller<B>, B: Len>: PackedStruct<B> 
{
    fn parse(data: &A) -> Result<Self, Error> {
        let mut ret = Self::unpack(data.resize())?;
        ret.verify()?;
        Ok(ret)
    }

    fn verify(&mut self) -> Result<(), Error> {
        Ok(())
    }
}
