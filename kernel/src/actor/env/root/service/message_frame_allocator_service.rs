use crate::{kprintln, log_dbg};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameAllocError {
    OutOfFrames,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameFreeError {
    OutOfRange,
    DoubleFree,
}

/// Simple bitmap-backed frame allocator for 2MiB frames.
/// bit=1 => allocated, bit=0 => free
pub struct MessageFrameAllocatorService {
    base: usize,
    bitmap: [u64; Self::BITMAP_WORDS],
}

impl MessageFrameAllocatorService {
    pub const FRAME_SIZE: usize = 0x200_000; // 2MiB
    pub const NUM_FRAMES: usize = 1000;
    const BITMAP_WORDS: usize = (Self::NUM_FRAMES + 63) / 64;

    pub const fn new(base: usize) -> Self {
        Self {
            base,
            bitmap: [0u64; Self::BITMAP_WORDS],
        }
    }

    #[inline]
    pub const fn base(&self) -> usize {
        self.base
    }

    #[inline]
    pub const fn frame_size(&self) -> usize {
        Self::FRAME_SIZE
    }

    #[inline]
    pub const fn capacity(&self) -> usize {
        Self::NUM_FRAMES
    }

    #[inline]
    pub const fn frame_addr(&self, id: usize) -> usize {
        self.base + id * Self::FRAME_SIZE
    }

    #[inline]
    pub fn alloc_frame(&mut self) -> Result<usize, FrameAllocError> {
        // First-fit scan of the bitmap words
        for word_idx in 0..Self::BITMAP_WORDS {
            let used = self.bitmap[word_idx];
            let free = !used;

            if free == 0 {
                continue;
            }

            let bit = free.trailing_zeros() as usize; // 0..63
            let id = word_idx * 64 + bit;

            if id >= Self::NUM_FRAMES {
                continue;
            }

            self.bitmap[word_idx] |= 1u64 << bit;
            return Ok(id);
        }

        Err(FrameAllocError::OutOfFrames)
    }

    #[inline]
    pub fn alloc_frame_addr(&mut self) -> Result<(usize, usize), FrameAllocError> {
        let id = self.alloc_frame()?;
        let addr = self.frame_addr(id);

        log_dbg!("[MSG_FRAME] Allocated frame: {id} @ {:#X}", addr);

        Ok((id, addr))
    }

    #[inline]
    pub fn free_frame(&mut self, id: usize) -> Result<(), FrameFreeError> {
        log_dbg!("[MSG_FRAME] Freeing frame: {id}");

        if id >= Self::NUM_FRAMES {
            return Err(FrameFreeError::OutOfRange);
        }

        let word_idx = id / 64;
        let bit = id % 64;
        let mask = 1u64 << bit;

        if (self.bitmap[word_idx] & mask) == 0 {
            return Err(FrameFreeError::DoubleFree);
        }

        self.bitmap[word_idx] &= !mask;
        Ok(())
    }
}

#[cfg(feature = "test")]
impl MessageFrameAllocatorService {
    pub fn self_test(&mut self) -> bool {
        fn fail(msg: &str) -> bool {
            kprintln!("[MSG_FRAME][SELFTEST][FAIL] {msg}");
            false
        }

        kprintln!(
            "[MSG_FRAME][SELFTEST] base={:#X} frame_size={} num_frames={}",
            self.base,
            Self::FRAME_SIZE,
            Self::NUM_FRAMES
        );

        // Ensure we start from a clean slate for the test.
        for w in self.bitmap.iter_mut() {
            *w = 0;
        }

        // 1) First alloc is id 0 and addr == base
        {
            let (id, addr) = match self.alloc_frame_addr() {
                Ok(v) => v,
                Err(_) => return fail("alloc_frame_addr failed on first allocation"),
            };
            kprintln!("[MSG_FRAME][SELFTEST] first alloc -> id={id}, addr={:#X}", addr);

            if id != 0 {
                return fail("first allocated id was not 0");
            }
            if addr != self.base {
                return fail("first allocated addr != base");
            }
        }

        // 2) Next few allocations are sequential and address increments by 2MiB
        {
            for expected_id in 1usize..10 {
                let (id, addr) = match self.alloc_frame_addr() {
                    Ok(v) => v,
                    Err(_) => return fail("alloc_frame_addr failed during sequential test"),
                };

                let expected_addr = self.base + expected_id * Self::FRAME_SIZE;
                if id != expected_id {
                    kprintln!(
                        "[MSG_FRAME][SELFTEST] expected id={expected_id}, got id={id}"
                    );
                    return fail("sequential id allocation failed");
                }
                if addr != expected_addr {
                    kprintln!(
                        "[MSG_FRAME][SELFTEST] expected addr={:#X}, got addr={:#X}",
                        expected_addr,
                        addr
                    );
                    return fail("sequential address calculation failed");
                }
            }
            kprintln!("[MSG_FRAME][SELFTEST] sequential alloc (0..10) OK");
        }

        // 3) Free and reuse: free 5 and ensure it's the next allocation (first-fit, lowest free)
        {
            if let Err(_) = self.free_frame(5) {
                return fail("free_frame(5) failed");
            }
            let id = match self.alloc_frame() {
                Ok(v) => v,
                Err(_) => return fail("alloc_frame failed when reusing a freed frame"),
            };
            kprintln!("[MSG_FRAME][SELFTEST] freed id=5 then alloc -> id={id}");
            if id != 5 {
                return fail("did not reuse freed id=5 (expected first-fit reuse)");
            }
        }

        // 4) Double-free detection
        {
            if let Err(_) = self.free_frame(5) {
                return fail("free_frame(5) failed (first time) for double-free test");
            }
            match self.free_frame(5) {
                Err(FrameFreeError::DoubleFree) => {
                    kprintln!("[MSG_FRAME][SELFTEST] double-free correctly detected");
                }
                Ok(_) => return fail("double-free was not detected"),
                Err(_) => return fail("unexpected error kind on double-free"),
            }
            // Re-allocate it to keep the state consistent-ish
            let id = match self.alloc_frame() {
                Ok(v) => v,
                Err(_) => return fail("alloc_frame failed after double-free test"),
            };
            if id != 5 {
                return fail("expected to re-allocate id=5 after freeing it");
            }
        }

        // 5) Out-of-range free
        {
            match self.free_frame(Self::NUM_FRAMES) {
                Err(FrameFreeError::OutOfRange) => {
                    kprintln!("[MSG_FRAME][SELFTEST] out-of-range free correctly detected");
                }
                _ => return fail("expected OutOfRange when freeing NUM_FRAMES"),
            }
        }

        // 6) Exhaustion: allocate remaining until OutOfFrames.
        //    We already allocated ids 0..9 (with some frees/reuses), but we can just allocate until failure.
        {
            let mut alloc_count: usize = 0;
            loop {
                match self.alloc_frame() {
                    Ok(_id) => alloc_count += 1,
                    Err(FrameAllocError::OutOfFrames) => break,
                }
            }

            // Total frames should be exactly NUM_FRAMES allocated at exhaustion.
            // We can compute "used" by summing popcounts.
            let mut used = 0usize;
            for &w in &self.bitmap {
                used += w.count_ones() as usize;
            }

            kprintln!(
                "[MSG_FRAME][SELFTEST] exhaustion reached: additional_allocs={} used_bits={}",
                alloc_count,
                used
            );

            if used != Self::NUM_FRAMES {
                return fail("exhaustion check failed: used bits != NUM_FRAMES");
            }

            // Further allocations must fail
            if self.alloc_frame().is_ok() {
                return fail("alloc_frame succeeded even though allocator should be exhausted");
            }
            if self.alloc_frame_addr().is_ok() {
                return fail("alloc_frame_addr succeeded even though allocator should be exhausted");
            }
        }

        // 7) Free the last frame and ensure it can be reallocated (checks tail bits handling)
        {
            let last = Self::NUM_FRAMES - 1;
            if let Err(_) = self.free_frame(last) {
                return fail("free_frame(last) failed");
            }
            let id = match self.alloc_frame() {
                Ok(v) => v,
                Err(_) => return fail("alloc_frame failed after freeing last frame"),
            };
            kprintln!(
                "[MSG_FRAME][SELFTEST] freed last id={} then alloc -> id={}",
                last,
                id
            );
            if id != last {
                return fail("expected to re-allocate the last frame id");
            }
        }

        kprintln!("[MSG_FRAME][SELFTEST][PASS] all checks passed");
        true
    }
}
