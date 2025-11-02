use crate::drivers::pl011::PL011;
use crate::hal::driver::Driver;
use crate::isr::ExceptionFrame;
use crate::kprintln;
use crate::platform::aarch64::registers::esr_el1::ESR_EL1;
use crate::platform::aarch64::registers::far_el1::FAR_EL1;
use core::arch::asm;

#[unsafe(no_mangle)]
extern "C" fn exc_serror(frame: &mut ExceptionFrame) {
    let mut lock = unsafe { PL011::new(0xFE201000) };
    let _ = lock.enable();

    let esr = ESR_EL1.read();
    let far = FAR_EL1.read();

    let ec = (esr >> 26) & 0x3f;
    let il = (esr >> 25) & 0x1;
    let iss = esr & 0x01ff_ffff;

    kprintln!("\n======== EL1 Exception========",);
    kprintln!("ESR_EL1 = {:#018x}", esr);
    kprintln!("  EC    = {:#04x}  ({})", ec, ec_str(ec));
    kprintln!("  IL    = {}", il);
    kprintln!("  ISS   = {:#08x}", iss);
    kprintln!("ELR_EL1 = {:#018x}", frame.elr_el1);
    kprintln!("SPSR_EL1= {:#018x}", frame.spsr_el1);
    kprintln!("FAR_EL1 = {:#018x}", far);

    match ec {
        0x24 | 0x25 => {
            // Data Abort
            let dfsc = iss & 0x3f;
            let wnr = (iss >> 6) & 1;
            let s1ptw = (iss >> 7) & 1;
            let cm = (iss >> 8) & 1;
            let ea = (iss >> 9) & 1;
            let fnv = (iss >> 10) & 1;
            kprintln!("-- Data Abort details --");
            kprintln!(
                "  WnR={} S1PTW={} CM={} EA={} FnV={}",
                wnr,
                s1ptw,
                cm,
                ea,
                fnv
            );
            kprintln!("  DFSC={:#04x} ({})", dfsc, dfsc_str(dfsc));
            if fnv == 0 {
                kprintln!("  FAR_EL1 valid: VA={:#018x}", far);
            } else {
                kprintln!("  FAR_EL1 not valid (FnV=1)");
            }
        }
        0x20 | 0x21 => {
            // Instruction Abort
            let ifsc = iss & 0x3f;
            kprintln!("-- Instruction Abort details --");
            kprintln!("  IFSC={:#04x} ({})", ifsc, ifsc_str(ifsc));
            let fnv = (iss >> 10) & 1;
            if fnv == 0 {
                kprintln!("  FAR_EL1 valid: VA={:#018x}", far);
            } else {
                kprintln!("  FAR_EL1 not valid (FnV=1)");
            }
        }
        0x2F => {
            // SError interrupt (asynchronous external abort)
            let aet = (iss >> 2) & 0b11; // if RAS is implemented
            kprintln!("-- SError details --");
            kprintln!("  AET={} (Architectural Error Type, if RAS present)", aet);
            kprintln!("  FAR may be unrelated/unknown for SError (platform-specific).");
        }
        _ => {
            kprintln!("(No specialized decoder for this EC).");
        }
    }

    // Dump all GPRs from the saved frame
    kprintln!("-- Registers --");
    kprintln!(
        "x0 ={:#018x}  x1 ={:#018x}  x2 ={:#018x}  x3 ={:#018x}",
        frame.x0,
        frame.x1,
        frame.x2,
        frame.x3
    );
    kprintln!(
        "x4 ={:#018x}  x5 ={:#018x}  x6 ={:#018x}  x7 ={:#018x}",
        frame.x4,
        frame.x5,
        frame.x6,
        frame.x7
    );
    kprintln!(
        "x8 ={:#018x}  x9 ={:#018x}  x10={:#018x}  x11={:#018x}",
        frame.x8,
        frame.x9,
        frame.x10,
        frame.x11
    );
    kprintln!(
        "x12={:#018x}  x13={:#018x}  x14={:#018x}  x15={:#018x}",
        frame.x12,
        frame.x13,
        frame.x14,
        frame.x15
    );
    kprintln!(
        "x16={:#018x}  x17={:#018x}  x18={:#018x}  x19={:#018x}",
        frame.x16,
        frame.x17,
        frame.x18,
        frame.x19
    );
    kprintln!(
        "x20={:#018x}  x21={:#018x}  x22={:#018x}  x23={:#018x}",
        frame.x20,
        frame.x21,
        frame.x22,
        frame.x23
    );
    kprintln!(
        "x24={:#018x}  x25={:#018x}  x26={:#018x}  x27={:#018x}",
        frame.x24,
        frame.x25,
        frame.x26,
        frame.x27
    );
    kprintln!(
        "x28={:#018x}  x29={:#018x}  x30={:#018x}",
        frame.x28,
        frame.x29,
        frame.x30
    );

    kprintln!("===============================\n");

    panic!("EL1_SERROR/ABORT");
}

#[inline(always)]
fn ec_str(ec: u64) -> &'static str {
    match ec {
        0x00 => "Unknown",
        0x15 => "SVC (AArch64)",
        0x20 => "Instr Abort (same EL)",
        0x21 => "Instr Abort (lower EL)",
        0x22 => "PC Alignment",
        0x24 => "Data Abort (same EL)",
        0x25 => "Data Abort (lower EL)",
        0x26 => "SP Alignment",
        0x2F => "SError interrupt",
        0x30 => "Breakpoint (lower EL)",
        0x31 => "Breakpoint (same EL)",
        _ => "Other/Reserved",
    }
}

#[inline(always)]
fn dfsc_str(dfsc: u64) -> &'static str {
    match dfsc {
        0x04 => "Translation fault, level 0",
        0x05 => "Translation fault, level 1",
        0x06 => "Translation fault, level 2",
        0x07 => "Translation fault, level 3",
        0x09 => "Access flag fault, level 1",
        0x0A => "Access flag fault, level 2",
        0x0B => "Access flag fault, level 3",
        0x0D => "Permission fault, level 1",
        0x0E => "Permission fault, level 2",
        0x0F => "Permission fault, level 3",
        0x10 => "Synchronous external abort",
        0x11 => "TLB conflict abort",
        0x18 => "Synchronous parity/ECC error",
        0x1F => "Implementation-defined fault (DFSC=0x1F)",
        _ => "Other/Reserved DFSC",
    }
}

#[inline(always)]
fn ifsc_str(ifsc: u64) -> &'static str {
    match ifsc {
        0x04 => "Translation fault, level 0",
        0x05 => "Translation fault, level 1",
        0x06 => "Translation fault, level 2",
        0x07 => "Translation fault, level 3",
        0x0D => "Permission fault, level 1",
        0x0E => "Permission fault, level 2",
        0x0F => "Permission fault, level 3",
        0x10 => "Synchronous external abort",
        0x18 => "Synchronous parity/ECC error",
        _ => "Other/Reserved IFSC",
    }
}
