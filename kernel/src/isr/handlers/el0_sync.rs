use crate::drivers::pl011::PL011;
use crate::hal::driver::Driver;
use crate::isr::Svc;
use crate::isr::context::{EL1Context, ISRContext};
use crate::{kprintln, log_dbg, log_dbg_naked};
use crate::platform::aarch64::cpu;
use core::arch::{asm, naked_asm};
use core::ptr::addr_of;
use core::slice;

fn el0_sys_write(ctx: *const ISRContext) {
    unsafe {
        let slice = slice::from_raw_parts((*ctx).x[0] as *const u8, (*ctx).x[1] as usize);
        kprintln!("User: {}", str::from_utf8_unchecked(slice));
    }
}

const EC_SVC64: u64 = 0x15;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn el0_sync(ctx: *const ISRContext, ctx_el1: *const EL1Context) {
    log_dbg!("EL0_SYNC");

    // Read ESR_EL1
    let esr: u64;
    unsafe {
        asm!(
        "mrs {0}, ESR_EL1",
        out(reg) esr,
        options(nomem, preserves_flags, nostack),
        );
    }

    let ec = (esr >> 26) & 0x3f;

    if ec != EC_SVC64 {
        el0_handler();
    }

    let svc_num = (esr & 0xffff) as u16;

    log_dbg_naked!("SVC_NUM: {:#X}", svc_num);

    match Svc::from(svc_num) {
        Svc::PrintMsg => {
            log_dbg_naked!("{:?}", unsafe{&*ctx});

            el0_sys_write(ctx);
        }
        Svc::ReturnEl1 => {
            log_dbg_naked!("RETURN_ADDR: {:#X}", unsafe {&*ctx_el1}.ret_addr as u64);
            log_dbg_naked!("{:?}", unsafe {&*ctx_el1});

            unsafe {
                asm!(
                    // Restore callee-saved regs from EL1Context
                    "ldp x29, x30, [x1, #32]",
                    "ldp x27, x28, [x1, #48]",
                    "ldp x25, x26, [x1, #64]",
                    "ldp x23, x24, [x1, #80]",
                    "ldp x21, x22, [x1, #96]",
                    "ldp x19, x20, [x1, #112]",

                    // Load resume PC and SP
                    "ldr x2, [x1, #16]",   // ret_addr
                    "msr ELR_EL1, x2",

                    "ldr x2, [x1, #24]",   // saved_sp
                    "mov sp, x2",

                    // Return to EL1h (pick DAIF mask as you want; this matches your earlier constant)
                    "mov x2, #( (1<<9) | (1<<8) | (0<<7) | (1<<6) | 0b0101 )",
                    "msr SPSR_EL1, x2",

                    "isb",
                    "eret",

                    in("x1") (ctx_el1 as u64),

                    options(noreturn),
                );
            }
        }
        Svc::SendMsg => {
            unimplemented!();
        }
        _ => {
            el0_handler();
        }
    }
}

const OFF_SAVED_SP: usize = 0x00;
const OFF_X19: usize = 0x08;
const OFF_LR: usize = 0x60;
const OFF_RESUME_PC: usize = 0x68;

fn el0_handler() {
    let mut uart = unsafe { PL011::new(0xFE201000) };

    let _ = uart.enable();

    let mut esr: u64;
    let mut far: u64;
    let mut elr: u64;
    let mut spsr: u64;

    unsafe {
        asm!(
        "mrs {esr},  esr_el1",
        "mrs {far},  far_el1",
        "mrs {elr},  elr_el1",
        "mrs {spsr}, spsr_el1",
        esr = out(reg) esr,
        far = out(reg) far,
        elr = out(reg) elr,
        spsr = out(reg) spsr,
        options(nomem, nostack, preserves_flags),
        );
    }

    let ec = ((esr >> 26) & 0x3f) as u32;
    let il = ((esr >> 25) & 0x1) != 0;
    let iss = (esr & 0x01ff_ffff) as u32;

    kprintln!("\n[EL0_SYNC] exception");
    kprintln!("  ELR_EL1  = {:#018x}", elr);
    kprintln!("  FAR_EL1  = {:#018x}", far);
    kprintln!(
        "  ESR_EL1  = {:#010x}  EC={:#04x}({}) IL={} ISS={:#08x}",
        esr as u32,
        ec,
        ec_to_str(ec),
        il as u8,
        iss
    );
    kprintln!(
        "  SPSR_EL1 = {:#018x}  NZCV={:04b}  D{} A{} I{} F{}  EL{}",
        spsr,
        ((spsr >> 28) & 0xF) as u8,
        ((spsr >> 9) & 1),
        ((spsr >> 8) & 1),
        ((spsr >> 7) & 1),
        ((spsr >> 6) & 1),
        ((spsr >> 2) & 0b11)
    );

    match ec {
        0x20 | 0x21 => {
            // Instruction abort (lower/same EL)
            let ifsc = iss & 0x3f;
            let s1ptw = (iss >> 7) & 1;
            let ea = (iss >> 9) & 1;
            let fnv = (iss >> 10) & 1;
            kprintln!(
                "  InstAbort: IFSC={:#04x} ({}) S1PTW={} EA={} FnV={}",
                ifsc,
                fs_to_str(ifsc),
                s1ptw,
                ea,
                fnv
            );
        }
        0x24 | 0x25 => {
            // Data abort (lower/same EL)
            let dfsc = iss & 0x3f;
            let wnr = (iss >> 6) & 1;
            let s1ptw = (iss >> 7) & 1;
            let cm = (iss >> 8) & 1;
            let ea = (iss >> 9) & 1;
            let fnv = (iss >> 10) & 1;
            kprintln!(
                "  DataAbort: DFSC={:#04x} ({}) WnR={} S1PTW={} CM={} EA={} FnV={}",
                dfsc,
                fs_to_str(dfsc),
                wnr,
                s1ptw,
                cm,
                ea,
                fnv
            );
        }
        0x15 => {
            // SVC from EL0
            let imm16 = iss & 0xffff;
            kprintln!("  SVC: imm16={:#06x}", imm16);
        }
        0x35 => {
            // BRK instruction
            let imm16 = iss & 0xffff;
            kprintln!("  BRK: imm16={:#06x}", imm16);
        }
        _ => { /* other ECs will still be visible via the raw ESR dump above */ }
    }

    loop {
        cpu::wfi();
    }
}

fn ec_to_str(ec: u32) -> &'static str {
    match ec {
        0x00 => "Unknown",
        0x15 => "SVC (EL0)",
        0x20 => "Instr abort (lower EL)",
        0x21 => "Instr abort (same EL)",
        0x24 => "Data abort (lower EL)",
        0x25 => "Data abort (same EL)",
        0x26 => "Alignment fault",
        0x28 => "FP/AdvSIMD",
        0x2C => "Breakpoint (lower EL)",
        0x2D => "Breakpoint (same EL)",
        0x30 => "Step (lower EL)",
        0x31 => "Step (same EL)",
        0x32 => "Watchpoint (lower EL)",
        0x33 => "Watchpoint (same EL)",
        0x35 => "BRK",
        _ => "Other",
    }
}

fn fs_to_str(fs: u32) -> &'static str {
    match fs {
        0x00 => "Addr size fault L0",
        0x01 => "Addr size fault L1",
        0x02 => "Addr size fault L2",
        0x03 => "Addr size fault L3",
        0x04 => "Translation fault L0",
        0x05 => "Translation fault L1",
        0x06 => "Translation fault L2",
        0x07 => "Translation fault L3",
        0x09 => "Access flag fault L1",
        0x0A => "Access flag fault L2",
        0x0B => "Access flag fault L3",
        0x0D => "Permission fault L1",
        0x0E => "Permission fault L2",
        0x0F => "Permission fault L3",
        0x10 => "Sync external abort",
        0x11 => "Async SError",
        0x14 => "TLB conflict",
        0x15 => "Unsupported atomic",
        0x21 => "Alignment fault",
        _ => "Unclassified",
    }
}
