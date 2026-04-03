use core::arch::asm;
use crate::kprintln;
use crate::platform::aarch64::cpu;
use crate::utils::stack_buf::StackBuf;

#[inline(always)]
fn read_esr_el1() -> u64 { let v; unsafe { core::arch::asm!("mrs {0}, esr_el1", out(reg) v); } v }
#[inline(always)]
fn read_elr_el1() -> u64 { let v; unsafe { core::arch::asm!("mrs {0}, elr_el1", out(reg) v); } v }
#[inline(always)]
fn read_far_el1() -> u64 { let v; unsafe { core::arch::asm!("mrs {0}, far_el1", out(reg) v); } v }
#[inline(always)]
fn read_spsr_el1() -> u64 { let v; unsafe { core::arch::asm!("mrs {0}, spsr_el1", out(reg) v); } v }

fn decode_esr(esr: u64) -> (u8, u32) {
    let ec  = ((esr >> 26) & 0x3f) as u8;
    let iss = (esr & 0x01ff_ffff) as u32;
    (ec, iss)
}

#[unsafe(no_mangle)]
extern "C" fn unhandled_irq() -> ! {
    cpu::write_daif(0xF);

    let esr  = read_esr_el1();
    let elr  = read_elr_el1();
    let far  = read_far_el1();
    let spsr = read_spsr_el1();
    let (ec, iss) = decode_esr(esr);

    let mut b = StackBuf::<512>::new();

    // Never unwrap in exception context; ignore fmt errors (we truncate anyway).
    let _ = core::fmt::write(&mut b, format_args!(
        "UNHANDLED EXCEPTION/IRQ\n\
         \tESR_EL1  = {:#018x} (EC={:#x}, ISS={:#x})\n\
         \tELR_EL1  = {:#018x}\n\
         \tFAR_EL1  = {:#018x}\n\
         \tSPSR_EL1 = {:#018x}\n",
        esr, ec, iss, elr, far, spsr
    ));

    kprintln!("{}", b.as_str());

    loop { unsafe { asm!("wfe"); } }
}