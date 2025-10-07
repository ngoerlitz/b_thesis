use core::ops::{BitAnd, BitOr, Not, Shl, Shr};

pub(crate) mod readonly;
pub(crate) mod register;
pub(crate) mod writeonly;

pub(crate) trait RegisterCommonBounds<T>:
    Copy
    + From<u8>
    + PartialEq
    + Shl<usize, Output = Self>
    + Shr<usize, Output = Self>
    + BitAnd<Output = Self>
    + BitOr<Output = Self>
    + Not<Output = Self>
{
}

impl<T> RegisterCommonBounds<T> for T where
    T: Copy
        + From<u8>
        + PartialEq
        + Shl<usize, Output = T>
        + Shr<usize, Output = T>
        + BitAnd<Output = T>
        + BitOr<Output = T>
        + Not<Output = T>
{
}
