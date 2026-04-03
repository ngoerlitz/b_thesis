//! BCM2711 / Raspberry Pi 4B reboot via PM watchdog/reset
//!
//! This matches the sequence used by the Linux bcm2835_wdt restart handler.

use core::ptr::{read_volatile, write_volatile};

/// ARM-visible peripheral base on Pi 4 (BCM2711) in "low peripherals" mode.
const PERIPHERAL_BASE: usize = 0xFE00_0000;

/// PM (power management + watchdog) block offset from peripheral base.
const PM_OFFSET: usize = 0x0010_0000;
const PM_BASE: usize = PERIPHERAL_BASE + PM_OFFSET;

/// Register offsets within PM block.
const PM_RSTC_OFFSET: usize = 0x1C;
const PM_RSTS_OFFSET: usize = 0x20;
const PM_WDOG_OFFSET: usize = 0x24;

/// Magic password required on writes to PM_* registers.
const PM_PASSWORD: u32 = 0x5A00_0000;

/// RSTC configuration bits.
const PM_RSTC_WRCFG_MASK: u32 = 0x0000_0030;
const PM_RSTC_WRCFG_FULL_RESET: u32 = 0x0000_0020;

/// Convert physical address to a volatile u32 pointer.
#[inline(always)]
const fn reg32(addr: usize) -> *mut u32 {
    addr as *mut u32
}

/// Trigger a full SoC reset.
///
/// The firmware/boot ROM will run again
pub fn reboot_via_watchdog() -> ! {
    unsafe {
        let rstc = reg32(PM_BASE + PM_RSTC_OFFSET);
        let _rsts = reg32(PM_BASE + PM_RSTS_OFFSET);
        let wdog = reg32(PM_BASE + PM_WDOG_OFFSET);

        write_volatile(wdog, PM_PASSWORD | 10);

        // Read current RSTC, clear the WRCFG bits, set FULL_RESET.
        let mut val = read_volatile(rstc);
        val &= !PM_RSTC_WRCFG_MASK;
        val |= PM_RSTC_WRCFG_FULL_RESET;
        val |= PM_PASSWORD;

        write_volatile(rstc, val);

        // Wait for reset to hit.
        loop {
            #[cfg(any(target_arch = "aarch64", target_arch = "arm"))]
            core::arch::asm!("wfe", options(nomem, nostack, preserves_flags));

            #[cfg(not(any(target_arch = "aarch64", target_arch = "arm")))]
            {
                // Fallback busy loop if building on a non-ARM host for tests
                core::hint::spin_loop();
            }
        }
    }
}
