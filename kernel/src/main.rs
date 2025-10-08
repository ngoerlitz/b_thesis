#![no_std]
#![no_main]
#![allow(unused, unused_variables)]
#![feature(allocator_api)]

extern crate alloc;

mod actor;
mod drivers;
mod ep_actor;
mod exc_vec;
mod future;
mod hal;
mod irq;
mod mmu;
mod platform;
mod root_actor;
mod test;

use crate::actor::root::actor_root_environment::{ActorRootEnvironment, ActorSpawnSpecification};
use crate::drivers::gic400::GIC400;
use crate::drivers::pl011::PL011;
use crate::ep_actor::EntryPointActor;
use crate::exc_vec::ExceptionFrame;
use crate::future::runtime::kernel_future_runtime_handler::{
    KernelFutureRuntime, KernelFutureRuntimeHandler,
};
use crate::hal::driver::Driver;
use crate::hal::irq::{CpuTarget, InterruptController};
use crate::hal::serial::{SerialDataBits, SerialDevice, SerialParity};
use crate::hal::timer::SystemTimer;
use crate::irq::{ExceptionLevel, IRQHandler};
use crate::mmu::{cause_data_translation_load, jump_to};
use crate::platform::aarch64::{cpu, get_cpu_timer};
use crate::root_actor::RootActor;
use crate::test::kernel_func;
use alloc::collections::btree_map::Entry;
use alloc::sync::Arc;
use core::arch::{asm, naked_asm};
use core::fmt::Write;
use core::ops::{Deref, DerefMut};
use core::panic::PanicInfo;
use core::time::Duration;
use linked_list_allocator::LockedHeap;
use spin::{Mutex, RwLock};
use zcene_core::actor::{ActorMessageSender, ActorSystem, ActorSystemReference};
use zcene_core::future::runtime::FutureRuntime;

#[inline(always)]
fn mmio_read32(p: usize) -> u32 {
    unsafe { core::ptr::read_volatile(p as *const u32) }
}
#[inline(always)]
fn mmio_write32(p: usize, v: u32) {
    unsafe { core::ptr::write_volatile(p as *mut u32, v) }
}

pub struct UartSink;
impl core::fmt::Write for UartSink {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let _ = UART0.lock().write_str(s);

        Ok(())
    }
}

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

unsafe extern "C" {
    static __heap_start: usize;
    static __heap_end: usize;
}

static UART0: Mutex<PL011> = Mutex::new(unsafe { PL011::new(0xFE201000) });
static IRQ: RwLock<IRQHandler> = RwLock::new(unsafe { IRQHandler::new(GIC400::new(0xFF84_0000)) });

pub fn init_heap() {
    unsafe {
        let start = (&__heap_start as *const _ as *const u8) as usize;
        let end = (&__heap_end as *const _ as *const u8) as usize;

        let heap_size = end - start;
        ALLOCATOR.lock().init(start as *mut u8, heap_size);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() -> ! {
    let cpuid = cpu::cpuid();

    if cpuid != 0 {
        {
            let _ = writeln!(UART0.lock(), "CORE: {}", cpuid);
        }

        test();

        // TODO: The secondary cores need to directly jump to the runtime's 'run-loop'
    }

    if cpuid == 0 {
        init_heap();

        let mut lock = UART0.lock();
        lock.set_baud_rate(48_000_000, 115_200);
        lock.set_parity(SerialParity::Even);
        lock.set_data_bits(SerialDataBits::Eight);
        let _ = lock.enable();

        let _ = writeln!(lock.deref_mut(), "UART initialized...");

        unsafe {
            let start = (&__heap_start as *const _ as *const u8) as usize;
            let end = (&__heap_end as *const _ as *const u8) as usize;

            let _ = writeln!(
                lock.deref_mut(),
                "[DEBUG]:: __heap_start: {:X} :: __heap_end: {:X} :: Size: {} Bytes ({} KiB - {} MiB - {} GiB)",
                start,
                end,
                end - start,
                (end - start) / 1024,
                (end - start) / (1024 * 1024),
                (end - start) / (1024 * 1024 * 1024),
            );
        }

        let mut irq = IRQ.write();
        irq.inner_mut().init();
        irq.inner_mut().debug(lock.deref_mut());
        irq.enable_irq(153, CpuTarget::Zero);

        irq.register_callback(153, |frame: &mut ExceptionFrame| {
            let mut lock = UART0.lock();

            match lock.read_byte() {
                Ok(char) => {
                    //let _ = writeln!(lock.deref_mut(), "READ_CHAR: {}", char as char);
                    let _ = write!(lock.deref_mut(), "{}", char as char);

                    if (char == 'j' as u8) {
                        unsafe { jump_to(0x4000_0000) };
                    }

                    if (char == 'd' as u8) {
                        unsafe { cause_data_translation_load(0x9000_0000u64) };
                    }

                    if (char == 'p' as u8) {
                        let mut gic = IRQ.write();
                        let _ = write!(lock.deref_mut(), "Setting new target: {}", unsafe {
                            (IRQ_CORE + 1 % 3)
                        });

                        unsafe {
                            if IRQ_CORE == 0 {
                                gic.set_irq_target_cpu(153, CpuTarget::One);
                                IRQ_CORE = 1;
                            } else if IRQ_CORE == 1 {
                                gic.set_irq_target_cpu(153, CpuTarget::Two);
                                IRQ_CORE = 2;
                            } else {
                                gic.set_irq_target_cpu(153, CpuTarget::Zero);
                                IRQ_CORE = 0;
                            }
                        }
                    }
                }
                Err(e) => {
                    let _ = writeln!(lock.deref_mut(), "NO CHAR FOUND! Err: {:?}", e);
                }
            }
        });

        irq.register_callback(30, |frame: &mut ExceptionFrame| {
            let timer = get_cpu_timer();
            timer.reset();
        });

        lock.enable_interrupt();
    }

    {
        let mut irq = IRQ.read();
        irq.inner().core_init();
    }

    let timer = get_cpu_timer();

    timer.init();
    timer.set_interval(Duration::from_secs(1));
    let _ = timer.enable();

    let mut counter = 0;

    {
        let mut lock = UART0.lock();
        let _ = writeln!(
            lock.deref_mut(),
            ">>>> [{}] SP = {:X}",
            cpuid,
            cpu::get_sp()
        );
    }

    // cpu::wake_secondary_cores();
    cpu::disable_irq();

    unsafe {
        mmu::init_enable_mmu_4k_l3_identity();
        mmu::intentionally_break();
    }
    cpu::enable_irq();

    kernel_func();

    let runtime = FutureRuntime::new(KernelFutureRuntimeHandler::default()).unwrap();

    let system: ActorSystemReference<ActorRootEnvironment<KernelFutureRuntimeHandler>> =
        ActorSystem::try_new(ActorRootEnvironment::new(runtime)).unwrap();

    system
        .spawn(ActorSpawnSpecification::new(RootActor))
        .unwrap();

    system.enter();

    // This runs the actual environment's runtime.

    loop {
        for _ in 0..100_000_00 {
            unsafe { asm!("nop") };
        }
    }
}

fn test() -> ! {
    {
        let mut irq = IRQ.read();
        irq.inner().core_init();
    }

    let timer = get_cpu_timer();

    timer.init();
    timer.set_interval(Duration::from_secs(1));
    let _ = timer.enable();

    cpu::enable_irq();

    loop {
        for _ in 0..100_000_00 {
            unsafe { asm!("nop") };
        }
    }
}

#[inline(always)]
fn read_sysreg_esr_el1() -> u64 {
    let v;
    unsafe { asm!("mrs {0}, ESR_EL1", out(reg) v, options(nomem, preserves_flags)) };
    v
}
#[inline(always)]
fn read_sysreg_far_el1() -> u64 {
    let v;
    unsafe { asm!("mrs {0}, FAR_EL1", out(reg) v, options(nomem, preserves_flags)) };
    v
}
#[inline(always)]
fn read_sysreg_pfar_el1() -> u64 {
    let v;
    unsafe { asm!("mrs {0}, PFAR_EL1", out(reg) v, options(nomem, preserves_flags)) };
    v
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

#[inline(always)]
fn mask_daif_all() {
    unsafe {
        // Mask Debug, SError, IRQ, FIQ (set DAIF bits)
        asm!(
            "msr daifset, #0b1111",
            "isb",
            options(nostack, preserves_flags)
        );
    }
}

#[unsafe(no_mangle)]
extern "C" fn el1_serror(frame: &mut ExceptionFrame) {
    let mut lock = unsafe { PL011::new(0xFE201000) };
    let _ = lock.enable();

    let _ = writeln!(lock, "1");

    let esr = read_sysreg_esr_el1(); // 0x8600000f -->
    let far = read_sysreg_far_el1(); // 0x28a00 -->

    let ec = (esr >> 26) & 0x3f;
    let il = (esr >> 25) & 0x1;
    let iss = esr & 0x01ff_ffff;

    let _ = writeln!(lock, "\n======== EL1 Exception ========");
    let _ = writeln!(lock, "ESR_EL1 = {:#018x}", esr);
    let _ = writeln!(lock, "  EC    = {:#04x}  ({})", ec, ec_str(ec));
    let _ = writeln!(lock, "  IL    = {}", il);
    let _ = writeln!(lock, "  ISS   = {:#08x}", iss);
    let _ = writeln!(lock, "ELR_EL1 = {:#018x}", frame.elr_el1);
    let _ = writeln!(lock, "SPSR_EL1= {:#018x}", frame.spsr_el1);
    let _ = writeln!(lock, "FAR_EL1 = {:#018x}", far);

    match ec {
        0x24 | 0x25 => {
            // Data Abort
            let dfsc = iss & 0x3f;
            let wnr = (iss >> 6) & 1;
            let s1ptw = (iss >> 7) & 1;
            let cm = (iss >> 8) & 1;
            let ea = (iss >> 9) & 1;
            let fnv = (iss >> 10) & 1;
            let _ = writeln!(lock, "-- Data Abort details --");
            let _ = writeln!(
                lock,
                "  WnR={} S1PTW={} CM={} EA={} FnV={}",
                wnr, s1ptw, cm, ea, fnv
            );
            let _ = writeln!(lock, "  DFSC={:#04x} ({})", dfsc, dfsc_str(dfsc));
            if fnv == 0 {
                let _ = writeln!(lock, "  FAR_EL1 valid: VA={:#018x}", far);
            } else {
                let _ = writeln!(lock, "  FAR_EL1 not valid (FnV=1)");
            }
        }
        0x20 | 0x21 => {
            // Instruction Abort
            let ifsc = iss & 0x3f;
            let _ = writeln!(lock, "-- Instruction Abort details --");
            let _ = writeln!(lock, "  IFSC={:#04x} ({})", ifsc, ifsc_str(ifsc));
            let fnv = (iss >> 10) & 1;
            if fnv == 0 {
                let _ = writeln!(lock, "  FAR_EL1 valid: VA={:#018x}", far);
            } else {
                let _ = writeln!(lock, "  FAR_EL1 not valid (FnV=1)");
            }
        }
        0x2F => {
            // SError interrupt (asynchronous external abort)
            let aet = (iss >> 2) & 0b11; // if RAS is implemented
            let _ = writeln!(lock, "-- SError details --");
            let _ = writeln!(
                lock,
                "  AET={} (Architectural Error Type, if RAS present)",
                aet
            );
            let _ = writeln!(
                lock,
                "  FAR may be unrelated/unknown for SError (platform-specific)."
            );
        }
        _ => {
            let _ = writeln!(lock, "(No specialized decoder for this EC).");
        }
    }

    // Dump all GPRs from the saved frame
    let _ = writeln!(lock, "-- Registers --");
    let _ = writeln!(
        lock,
        "x0 ={:#018x}  x1 ={:#018x}  x2 ={:#018x}  x3 ={:#018x}",
        frame.x0, frame.x1, frame.x2, frame.x3
    );
    let _ = writeln!(
        lock,
        "x4 ={:#018x}  x5 ={:#018x}  x6 ={:#018x}  x7 ={:#018x}",
        frame.x4, frame.x5, frame.x6, frame.x7
    );
    let _ = writeln!(
        lock,
        "x8 ={:#018x}  x9 ={:#018x}  x10={:#018x}  x11={:#018x}",
        frame.x8, frame.x9, frame.x10, frame.x11
    );
    let _ = writeln!(
        lock,
        "x12={:#018x}  x13={:#018x}  x14={:#018x}  x15={:#018x}",
        frame.x12, frame.x13, frame.x14, frame.x15
    );
    let _ = writeln!(
        lock,
        "x16={:#018x}  x17={:#018x}  x18={:#018x}  x19={:#018x}",
        frame.x16, frame.x17, frame.x18, frame.x19
    );
    let _ = writeln!(
        lock,
        "x20={:#018x}  x21={:#018x}  x22={:#018x}  x23={:#018x}",
        frame.x20, frame.x21, frame.x22, frame.x23
    );
    let _ = writeln!(
        lock,
        "x24={:#018x}  x25={:#018x}  x26={:#018x}  x27={:#018x}",
        frame.x24, frame.x25, frame.x26, frame.x27
    );
    let _ = writeln!(
        lock,
        "x28={:#018x}  x29={:#018x}  x30={:#018x}",
        frame.x28, frame.x29, frame.x30
    );

    let _ = writeln!(lock, "===============================\n");

    panic!("EL1_SERROR/ABORT");
}

const OFF_SAVED_SP: usize = 0x00;
const OFF_X19: usize = 0x08;
const OFF_LR: usize = 0x60;
const OFF_RESUME_PC: usize = 0x68;

#[unsafe(no_mangle)]
unsafe extern "C" fn el0_handler() {
    let mut uart = unsafe { PL011::new(0xFE201000) };

    let _ = uart.enable();

    let mut esr: u64;
    let mut far: u64;
    let mut elr: u64;
    let mut spsr: u64;

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

    let ec = ((esr >> 26) & 0x3f) as u32;
    let il = ((esr >> 25) & 0x1) != 0;
    let iss = (esr & 0x01ff_ffff) as u32;

    let _ = writeln!(uart, "\n[EL0_SYNC] exception");
    let _ = writeln!(uart, "  ELR_EL1  = {:#018x}", elr);
    let _ = writeln!(uart, "  FAR_EL1  = {:#018x}", far);
    let _ = writeln!(
        uart,
        "  ESR_EL1  = {:#010x}  EC={:#04x}({}) IL={} ISS={:#08x}",
        esr as u32,
        ec,
        ec_to_str(ec),
        il as u8,
        iss
    );
    let _ = writeln!(
        uart,
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
            let _ = writeln!(
                uart,
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
            let _ = writeln!(
                uart,
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
            let _ = writeln!(uart, "  SVC: imm16={:#06x}", imm16);
        }
        0x35 => {
            // BRK instruction
            let imm16 = iss & 0xffff;
            let _ = writeln!(uart, "  BRK: imm16={:#06x}", imm16);
        }
        _ => { /* other ECs will still be visible via the raw ESR dump above */ }
    }

    // Optional: also dump the PTEs for ELR/FAR using your existing helper(s).
    // dump_pte_for_va(elr as usize);
    // dump_pte_for_va(far as usize);

    // Halt here so you can read the output.
    loop {
        asm!("wfi", options(nomem, nostack, preserves_flags));
    }
}

#[unsafe(no_mangle)]
#[unsafe(naked)]
pub unsafe extern "C" fn el0_sync() -> ! {
    // This also handles SVC -> therefore needed to handle EL0 -> EL1 transition

    naked_asm!(
        "mrs x0, ESR_EL1",
        "ubfx x1, x0, #26, #6", // EC
        "cmp x1, #0x15",
        "b.ne el0_handler", // not SVC -> default handler
        // Optionally check the immediate:
        "and x1, x0, #0xFFFF",
        "cmp x1, #0x10",
        "b.ne el0_handler",
        "ldr x0, [sp, #16 * 17]", // x2 = x_ptr
        "ldr w3, [x0]",           // *x2
        "add w3, w3, #1",
        "str w3, [x0]",
        "dsb ishst",
        "isb",
        // Restore registers
        "ldp x2, x3,   [sp, #16 * 1]",
        "ldp x4, x5,   [sp, #16 * 2]",
        "ldp x6, x7,   [sp, #16 * 3]",
        "ldp x8, x9,   [sp, #16 * 4]",
        "ldp x10, x11, [sp, #16 * 5]",
        "ldp x12, x13, [sp, #16 * 6]",
        "ldp x14, x15, [sp, #16 * 7]",
        "ldp x16, x17, [sp, #16 * 8]",
        "ldp x18, x19, [sp, #16 * 9]",
        "ldp x20, x21, [sp, #16 * 10]",
        "ldp x22, x23, [sp, #16 * 11]",
        "ldp x24, x25, [sp, #16 * 12]",
        "ldp x26, x27, [sp, #16 * 13]",
        "ldp x28, x29, [sp, #16 * 14]",
        "ldr x30, [sp, #16 * 16]",
        "mov x0, #(1<<9 | 1<<8 | 1<<7 | 1<<6 | 0b0101)", // DAIF = 1111, M=EL1h,
        "msr SPSR_EL1, x0",
        "msr ELR_EL1, x30",
        // Since we clobbered x1 in the mov call above, we need to load them down here.
        "ldp x0, x1,   [sp, #16 * 0]",
        "add sp, sp, #288",
        // Return
        "eret",
    )
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

#[unsafe(no_mangle)]
extern "C" fn err_invalid() {
    let mut lock = UART0.lock();
    let _ = writeln!(lock.deref_mut(), "ERR_INVALID");
    loop {}
}

#[unsafe(no_mangle)]
extern "C" fn el0_irq(exception_frame: &mut ExceptionFrame) {
    let mut irq = IRQ.read();
    irq.handle(exception_frame, ExceptionLevel::User);
}

static mut IRQ_CORE: u8 = 0;

#[unsafe(no_mangle)]
extern "C" fn el1_irq(exception_frame: &mut ExceptionFrame) {
    let mut irq = IRQ.read();
    irq.handle(exception_frame, ExceptionLevel::Kernel);
}

#[unsafe(no_mangle)]
extern "C" fn unhandled_irq() {
    let mut lock = UART0.lock();
    let _ = writeln!(lock.deref_mut(), "UNHANDLED_IRQ");
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    cpu::disable_irq();
    let core_id = cpu::cpuid();

    // Here, we re-initialize the PL011, since the lock may still be held by a different
    // core. We have to accept the fact that there may be some weird things going on when writing
    // concurrently - ideally, we'd use UART1-3 for this, but that's not emulated by QEMU iirc...
    // Otherwise, we'd not get ANY output, which is arguably worse :D
    let mut uart = unsafe { PL011::new(0xFE201000) };

    let _ = uart.enable();

    {
        let _ = writeln!(uart, "[Core: {}] Kernel Panic!\n{}", core_id, info);
    }
    loop {}
}
