use crate::drivers::common::RegisterCommonBounds;
use crate::drivers::common::register::Register;
use core::ops::Deref;

#[repr(transparent)]
pub struct ReadOnly<R>(R);

impl<T> ReadOnly<Register<T>>
where
    T: RegisterCommonBounds<T>,
{
    #[inline(always)]
    pub fn read(&self) -> T {
        self.0.read()
    }

    #[inline(always)]
    pub fn read_bit(&self, idx: usize) -> bool {
        self.0.read_bit(idx)
    }
}

impl<T> Deref for ReadOnly<Register<T>> {
    type Target = Register<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
