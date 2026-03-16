use core::arch::asm;

pub mod k2k;
pub mod u2u;
pub mod k2u;

pub fn get_time() -> (u64, u64) {
    let ticks: u64;
    let hz: u64;

    unsafe {
        asm!("mrs {ticks}, cntpct_el0",
            "mrs {hz}, cntfrq_el0",
            ticks = out(reg) ticks,
            hz = out(reg) hz,
            options(nomem, nostack, preserves_flags)
        )
    }

    (ticks, hz)
}

pub fn sleep(c: u64) {
    for i in 0..c {
        unsafe {
            asm!("nop");
        }
    }
}