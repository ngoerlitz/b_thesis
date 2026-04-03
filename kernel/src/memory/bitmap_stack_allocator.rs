use crate::{log_dbg, kprintln};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllocError {
    OutOfStacks,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FreeError {
    OutOfRange,
    DoubleFree,
}

/// bit=1 => allocated, bit=0 => free
pub struct StackAllocator {
    base: usize,
    stack_size: usize,
    bitmap: u64,
}

impl StackAllocator {
    /// Stacks are laid out downward from `base`:
    ///   id 0 top = base
    ///   id 1 top = base - stack_size
    ///   ...
    pub const fn new(base: usize, stack_size: usize) -> Self {
        Self {
            base,
            stack_size,
            bitmap: 0,
        }
    }

    #[inline]
    pub const fn stack_size(&self) -> usize {
        self.stack_size
    }

    #[inline]
    pub fn get_stack(&mut self) -> Result<usize, AllocError> {
        let free = !self.bitmap;
        if free == 0 {
            return Err(AllocError::OutOfStacks);
        }

        let bit = free.trailing_zeros() as usize; // 0..63
        self.bitmap |= 1u64 << bit;
        Ok(bit)
    }

    #[inline]
    pub fn get_stack_addr(&mut self) -> Result<(usize, usize), AllocError> {
        let id = self.get_stack()?;
        let addr = self.base - (id * self.stack_size);

        log_dbg!("GOT STACK ADDR: {} --> {:#X}", id, addr);

        Ok((id, addr))
    }

    #[inline]
    pub fn free_stack(&mut self, id: usize) -> Result<(), FreeError> {
        log_dbg!("[USER_STACK] Freeing stack: {id}");

        if id >= 64 {
            return Err(FreeError::OutOfRange);
        }

        let mask = 1u64 << id;
        if (self.bitmap & mask) == 0 {
            return Err(FreeError::DoubleFree);
        }

        self.bitmap &= !mask;
        Ok(())
    }
}