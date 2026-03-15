use core::arch::asm;

pub mod _1_2x_k2k_100_bytes_copy;
pub mod _1_2x_k2k_100_bytes_move;

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