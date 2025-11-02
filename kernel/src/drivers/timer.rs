use crate::hal::driver::Driver;
use crate::hal::timer::SystemTimer;
use crate::kprintln;
use crate::platform::aarch64::registers::cntfrq_el0::CNTFRQ_EL0;
use crate::platform::aarch64::registers::cntp_ctl_el0::CNTP_CTL_EL0;
use crate::platform::aarch64::registers::cntp_tval_el0::CNTP_TVAL_EL0;
use crate::platform::aarch64::registers::cntpct_el0::CNTPCT_EL0;
use core::fmt::{Display, Formatter, Write};
use core::time::Duration;

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
    }

    fn get_timer_info(&self) -> (u64, u64) {
        (CNTFRQ_EL0.read(), CNTPCT_EL0.read())
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

impl Display for EL1PhysicalTimer {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let freq = CNTFRQ_EL0.read();
        let ctl = CNTPCT_EL0.read();
        let count = CNTPCT_EL0.read();

        writeln!(f, "=== TIMER DEBUG ===");
        writeln!(f, "Frequency: {} Hz", freq);
        writeln!(
            f,
            "Control: 0b{:03b} (ENABLE={}, IMASK={}, ISTATUS={})",
            ctl,
            ctl & 1,
            (ctl >> 1) & 1,
            (ctl >> 2) & 1
        );
        writeln!(f, "Current count: {}", count);

        // Check if timer is actually firing
        if (ctl >> 2) & 1 != 0 {
            writeln!(f, "Timer condition is MET (ISTATUS=1)");
        } else {
            writeln!(f, "Timer condition not met (ISTATUS=0)");
        }

        if (ctl >> 1) & 1 != 0 {
            writeln!(f, "Timer interrupt is MASKED (IMASK=1)");
        } else {
            writeln!(f, "Timer interrupt is UNMASKED (IMASK=0)");
        }

        Ok(())
    }
}

impl SystemTimer for EL1PhysicalTimer {
    fn get_value(&self) -> u64 {
        CNTPCT_EL0.read()
    }

    fn get_frequency(&self) -> u64 {
        self.get_timer_info().0
    }
}

impl Driver for EL1PhysicalTimer {
    const NAME: &'static str = "EL1-Physical Timer - Timer Driver";

    fn enable(&mut self) -> Result<(), ()> {
        CNTP_CTL_EL0.write(CNTP_CTL_EL0::ENABLE);

        Ok(())
    }

    fn disable(&mut self) {
        todo!()
    }
}
