use crate::getter;
use core::fmt::{Display, Formatter};
use core::ptr::write;

pub trait SystemTimerDriver {
    fn now(&self) -> Instant;
    fn frequency(&self) -> u64;
}

pub struct Instant {
    ticks: u64,
    frequency: u64,
}

impl Display for Instant {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Instant")
            .field("ticks", &self.ticks)
            .field("frequency", &self.frequency)
            .finish()
    }
}

impl Instant {
    #[inline]
    pub const fn from_ticks(ticks: u64, frequency: u64) -> Self {
        Self { ticks, frequency }
    }

    getter!(ticks: u64);

    #[inline]
    pub fn as_secs(&self) -> u64 {
        self.ticks / self.frequency
    }

    #[inline]
    pub fn as_millis(&self) -> u64 {
        ((self.ticks as u128) * 1000u128 / (self.frequency as u128)) as u64
    }

    #[inline]
    pub fn as_micros(&self) -> u64 {
        ((self.ticks as u128) * 1_000_000u128 / (self.frequency as u128)) as u64
    }
}
