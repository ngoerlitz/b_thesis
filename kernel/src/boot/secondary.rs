use crate::boot::CpuBootInformation;
use crate::platform::aarch64::cpu;
use core::arch::asm;
use core::fmt::Write;

#[unsafe(no_mangle)]
pub extern "C" fn kernel_secondary(arg0: &mut CpuBootInformation) {
    let cpuid = cpu::cpuid();
    let mut sp: u64;
    unsafe {
        asm!("mov {}, sp", out(reg) sp);
    }

    loop {
        {
            let mut lock = arg0.uart.lock();
            writeln!(
                lock,
                "CPUID: {:X} | RandVal: {:3} | SP: {:#0X} ---- {:?}",
                cpuid, arg0.rand_value, sp, arg0
            );
        }

        for _ in 0..5_000_000 {
            unsafe {
                asm!("nop");
            }
        }
    }
}
