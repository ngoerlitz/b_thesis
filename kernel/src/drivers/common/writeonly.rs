use crate::drivers::common::RegisterCommonBounds;
use crate::drivers::common::register::Register;
use core::ops::{Deref, DerefMut};

#[repr(transparent)]
pub(crate) struct WriteOnly<R>(R);

impl<T> WriteOnly<Register<T>>
where
    T: RegisterCommonBounds<T>,
{
    #[inline(always)]
    pub fn write(&mut self, value: T) {
        self.0.write(value);
    }

    #[inline(always)]
    pub fn modify(&mut self, f: impl FnOnce(T) -> T) {
        self.0.modify(f);
    }

    #[inline(always)]
    pub fn zero(&mut self) {
        self.0.zero();
    }

    pub fn write_bit(&mut self, idx: usize, val: bool) {
        self.0.write_bit(idx, val);
    }
}

impl<T> Deref for WriteOnly<Register<T>> {
    type Target = Register<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for WriteOnly<Register<T>> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
