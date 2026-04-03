use crate::hal::driver::Driver;
use crate::hal::timer::{Instant, SystemTimerDriver};
use crate::kprintln;
use crate::platform::aarch64::registers::cntfrq_el0::CNTFRQ_EL0;
use crate::platform::aarch64::registers::cntp_ctl_el0::CNTP_CTL_EL0;
use crate::platform::aarch64::registers::cntp_cval_el0::CNTP_CVAL_EL0;
use crate::platform::aarch64::registers::cntp_tval_el0::CNTP_TVAL_EL0;
use crate::platform::aarch64::registers::cntpct_el0::CNTPCT_EL0;
use core::fmt::{Display, Formatter, Write};
use core::time::Duration;
use crate::platform::aarch64::registers::cntkctl_el1::CNTKCTL_EL1;

#[derive(Copy, Clone)]
pub struct EL1PhysicalTimer {
    duration: Option<Duration>,
    frq: Option<u64>,
}

impl EL1PhysicalTimer {
    pub const fn new() -> Self {
        Self {
            duration: None,
            frq: None,
        }
    }

    pub fn init(&mut self) {
        self.frq = Some(CNTFRQ_EL0.read());
        CNTP_CTL_EL0.clear_bit(0);

        CNTKCTL_EL1.enable_bit(0);
    }

    pub fn mask_irq(&mut self) {
        CNTP_CTL_EL0.enable_bit(CNTP_CTL_EL0::BIT_IMASK);
    }

    pub fn unmask_irq(&mut self) {
        CNTP_CTL_EL0.clear_bit(CNTP_CTL_EL0::BIT_IMASK);
    }

    fn get_timer_details(&self) -> (u64, u64) {
        (CNTPCT_EL0.read(), CNTFRQ_EL0.read())
    }

    pub fn set_interval(&mut self, duration: Duration) {
        self.duration = Some(duration);
    }

    pub fn reset(&mut self) {
        if let Some(duration) = self.duration {
            let freq = CNTFRQ_EL0.read() as u128; // ticks per second
            let nanos = duration.as_nanos() as u128;

            // ticks = round((freq * nanos) / 1e9)
            let mut ticks = (freq * nanos + 999_999_999) / 1_000_000_000;

            // TVAL is a signed 32-bit interval; clamp and ensure at least 1 tick
            if ticks == 0 {
                ticks = 1;
            }
            if ticks > i32::MAX as u128 {
                ticks = i32::MAX as u128;
            }

            CNTP_TVAL_EL0.write(ticks as u32 as u64);
        }
    }
}

impl SystemTimerDriver for EL1PhysicalTimer {
    fn now(&self) -> Instant {
        let info = self.get_timer_details();
        Instant::from_ticks(info.0, info.1)
    }

    fn frequency(&self) -> u64 {
        self.get_timer_details().1
    }
}

impl Driver for EL1PhysicalTimer {
    const NAME: &'static str = "EL1-Physical Timer - Timer Driver";

    fn enable(&mut self) -> Result<(), ()> {
        CNTP_CTL_EL0.write(CNTP_CTL_EL0::ENABLE);

        Ok(())
    }

    fn disable(&mut self) {
        CNTP_CTL_EL0.write(CNTP_CTL_EL0::DISABLE);
    }
}

impl Display for EL1PhysicalTimer {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let freq = CNTFRQ_EL0.read(); // frequency (Hz)
        let cnt = CNTPCT_EL0.read(); // current counter
        let tval = CNTP_TVAL_EL0.read(); // remaining ticks (signed)
        let cval = CNTP_CVAL_EL0.read(); // absolute compare value
        let ctl = CNTP_CTL_EL0.read(); // control/flags

        let enable = ctl & 1;
        let imask = (ctl >> 1) & 1;
        let istatus = (ctl >> 2) & 1;

        writeln!(f, "================ EL1 TIMER DEBUG ================")?;
        writeln!(f, " Frequency     : {} Hz", freq)?;
        writeln!(f, " Control (raw) : 0b{:b}", ctl)?;
        writeln!(f, " Current Count : {}", cnt)?;
        writeln!(f, " CVAL          : {}", cval)?;
        writeln!(f, " TVAL          : {}", tval as i64)?;
        writeln!(f)?;
        writeln!(f, " Flags:")?;
        writeln!(
            f,
            "   ENABLE  = {}   {}",
            enable,
            if enable == 1 {
                "(active)"
            } else {
                "(disabled)"
            }
        )?;
        writeln!(
            f,
            "   IMASK   = {}   {}",
            imask,
            if imask == 1 {
                "(interrupt masked)"
            } else {
                "(interrupt unmasked)"
            }
        )?;
        writeln!(
            f,
            "   ISTATUS = {}   {}",
            istatus,
            if istatus == 1 {
                "(condition met)"
            } else {
                "(condition not met)"
            }
        )?;
        writeln!(f, "================================================")?;
        Ok(())
    }
}
