use core::arch::asm;

pub(crate) fn cpuid() -> u8 {
    let mpidr_el1: u64;
    unsafe {
        asm!("mrs {mpidr_el1}, mpidr_el1", mpidr_el1 = out(reg) mpidr_el1, options(nostack, preserves_flags));
    }

    (mpidr_el1 & 0x3) as u8
}

pub(crate) fn current_el() -> &'static str {
    let mut el: u8;
    unsafe {
        asm!("mrs {el}, CurrentEL", el = out(reg) el);
    }

    match (el & 0b1100) >> 2 {
        0b00 => "EL0",
        0b01 => "EL1",
        0b10 => "EL2",
        0b11 => "EL3",
        _ => unreachable!(),
    }
}

/// Wakes all cores currently waiting for an event (WFE)
pub(crate) fn wake_secondary_cores() {
    unsafe {
        asm!(
            "adrp x1, WAKEUP_FLAG",
            "mov w2, #1",
            "str w2, [x1, :lo12:WAKEUP_FLAG]",
            "dsb ishst",
            "sev",
            options(nostack, preserves_flags)
        );
    }
}

pub(crate) fn get_sp() -> u64 {
    let sp: u64;
    unsafe {
        asm!("mov {}, sp", out(reg) sp, options(nostack, preserves_flags));
    }

    sp
}

pub(crate) fn enable_irq() {
    unsafe {
        asm!("msr daifclr, #0b111");
    }
}

pub(crate) fn disable_irq() {
    unsafe {
        asm!("msr daifset, #0b111");
    }
}
