use crate::drivers::common::RegisterCommonBounds;

#[repr(transparent)]
pub(crate) struct Register<T> {
    value: T,
}

impl<T> Register<T>
where
    T: RegisterCommonBounds<T>,
{
    #[inline(always)]
    #[allow(unused)]
    pub(crate) fn read(&self) -> T {
        unsafe { core::ptr::read_volatile(&self.value) }
    }

    #[inline(always)]
    #[allow(unused)]
    pub(crate) fn write(&mut self, value: T) {
        unsafe { core::ptr::write_volatile(&mut self.value, value) }
    }

    #[inline(always)]
    #[allow(unused)]
    pub(crate) fn modify(&mut self, f: impl FnOnce(T) -> T) {
        let curr = self.read();
        self.write(f(curr));
    }

    #[inline(always)]
    #[allow(unused)]
    pub(crate) fn zero(&mut self) {
        self.write(T::from(0u8));
    }

    #[inline(always)]
    #[allow(unused)]
    fn enable_bit(&mut self, idx: usize) {
        debug_assert!(idx < size_of::<T>() * 8);
        let one = T::from(1u8);
        self.modify(|v| v | (one << idx));
    }

    #[inline(always)]
    #[allow(unused)]
    fn clear_bit(&mut self, idx: usize) {
        debug_assert!(idx < size_of::<T>() * 8);
        let one = T::from(1u8);
        self.modify(|v| v & !(one << idx));
    }

    #[inline(always)]
    #[allow(unused)]
    pub fn write_bit(&mut self, idx: usize, val: bool) {
        if val {
            self.enable_bit(idx)
        } else {
            self.clear_bit(idx)
        }
    }

    #[inline(always)]
    #[allow(unused)]
    pub(crate) fn read_bit(&self, idx: usize) -> bool {
        debug_assert!(idx < size_of::<T>() * 8);
        let one = T::from(1u8);
        ((self.read() >> idx) & one) == one
    }
}

pub type RegU32 = Register<u32>;
pub type RegU64 = Register<u64>;
