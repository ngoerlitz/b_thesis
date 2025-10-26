#![no_std]
#![no_main]
#![allow(unused, unused_variables)]
#![feature(allocator_api)]
#![feature(format_args_nl)]

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
use core::fmt::{Debug, Display, Formatter, Write};
use core::ops::{Deref, DerefMut};
use core::panic::PanicInfo;
use core::time::Duration;
use core::{fmt, ptr};
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

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

unsafe extern "C" {
    static __heap_start: usize;
    static __heap_end: usize;
}

static UART0: Mutex<PL011> = Mutex::new(unsafe { PL011::new(0xFE201000) });
static IRQ: RwLock<IRQHandler> = RwLock::new(unsafe { IRQHandler::new(GIC400::new(0xFF84_0000)) });

pub fn kprint(args: fmt::Arguments) {
    cpu::with_irq_masked(|| {
        let mut guard = UART0.lock();
        let _ = fmt::write(&mut *guard, args);
    });
}

#[macro_export]
macro_rules! kprint {
    ($($arg:tt)*) => {
        $crate::kprint(format_args!($($arg)*));
    }
}
#[macro_export]
macro_rules! kprintln {
    ($($arg:tt)*) => {{
        $crate::kprint(core::format_args_nl!($($arg)*));
    }}
}

pub struct UartSink;
impl core::fmt::Write for UartSink {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        return Ok(());

        cpu::disable_irq();
        {
            let mut lock = UART0.lock();
            lock.write_str(s);
        }
        cpu::enable_irq();

        Ok(())
    }
}

#[macro_export]
macro_rules! linker_symbols {
    (
        $(
            $name:ident = $linker_sym:ident ;
        )*
    ) => {
        $(
            unsafe extern "C" {
                // We declare the linker symbol as if it were an extern static.
                // We only ever take its address; we never read/write it.
                //
                // Type here: u8 is conventional because it's just "a byte at that address".
                // You could make this configurable, but u8 is correct for “label points here”.
                static $linker_sym: u8;
            }

            #[allow(non_snake_case)]
            pub fn $name() -> usize {
                // SAFETY: We're just taking the address of a linker-defined symbol.
                // This does not dereference it.
                unsafe { core::ptr::addr_of!($linker_sym) as usize}
            }
        )*
    }
}

linker_symbols! {
    MAILBOX_TOP = __mailbox_top;
    KSTACK_01_TOP = __stack_01_el1_top;
    KSTACK_02_TOP = __stack_02_el1_top;
    KSTACK_03_TOP = __stack_03_el1_top;
}

pub fn init_heap() {
    unsafe {
        let start = (&__heap_start as *const _ as *const u8) as usize;
        let end = (&__heap_end as *const _ as *const u8) as usize;

        let heap_size = end - start;
        ALLOCATOR.lock().init(start as *mut u8, heap_size);
    }
}

#[repr(C)]
struct CpuBootInformation {
    uart: Arc<Mutex<PL011>>,
    rand_value: u64,
}

impl Debug for CpuBootInformation {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("CpuBootInformation")
            .field("rand_value", &self.rand_value)
            .finish()
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
struct CpuMailbox {
    sp: u64,
    init_func: u64,
    arg0: u64,
    go: u64,
}

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

/// Write data to CPU's mailbox (for wake-up configuration)
/// Note that QEMU does **NOT** emulate the `wfe` instruction and instead treats it as a `nop`.
/// This means that any `wfe` is subsequently ignored and all loops building on this requirement
/// are all busy-waited!
pub unsafe fn write_mailbox_for_core(core_index: u8, sp: u64, arg0: u64) {
    let base = MAILBOX_TOP();

    let mailbox_addr = base + (core_index as usize) * core::mem::size_of::<CpuMailbox>();

    let mbox_ptr = mailbox_addr as *mut CpuMailbox;

    let mbox_val = CpuMailbox {
        sp,
        init_func: kernel_secondary as *const () as u64,
        arg0,
        go: 1u64,
    };

    ptr::write_volatile(mbox_ptr, mbox_val);
}

fn init_mailboxes() {
    let base = MAILBOX_TOP() as usize;

    for core_id in 0..4 {
        let addr = base + (core_id * core::mem::size_of::<CpuMailbox>()) as usize;
        let ptr = addr as *mut CpuMailbox;

        // sp/arg0 don't matter yet, go=0 means "stay parked"
        unsafe {
            core::ptr::write_volatile(
                ptr,
                CpuMailbox {
                    sp: 0,
                    init_func: 0x80_000,
                    arg0: 0,
                    go: 0,
                },
            );
        }
    }
}

fn get_core_stack(core_id: u8) -> usize {
    match core_id {
        1 => KSTACK_01_TOP(),
        2 => KSTACK_02_TOP(),
        3 => KSTACK_03_TOP(),
        _ => unimplemented!(),
    }
}

unsafe fn write_cpu_boot_info(core_id: u8, info: CpuBootInformation) -> (u64, u64) {
    // 1. Top of this core's stack region
    let stack_top = get_core_stack(core_id) as usize;

    // 2. Place CpuBootInformation at the very top
    let info_size = core::mem::size_of::<CpuBootInformation>();
    let info_base = stack_top - info_size;

    // 3. Actually write it
    core::ptr::write_volatile(info_base as *mut CpuBootInformation, info);

    // 4. Compute initial SP for the core
    //    Go 16 bytes below info_base, then align down to 16.
    let tmp = info_base - 16;
    let sp_aligned = tmp & !0xFusize; // clear low 4 bits -> multiple of 16

    // 5. Values to hand to the secondary core:
    //    x0 = &CpuBootInformation
    //    sp = aligned stack pointer
    (info_base as u64, sp_aligned as u64)
}

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main() -> ! {
    init_heap();
    init_mailboxes();

    let serial_device = unsafe { PL011::new(0xFE201000) };
    let shared_device: Arc<Mutex<PL011>> = Arc::new(Mutex::new(serial_device));

    unsafe {
        let (arg0, sp) = write_cpu_boot_info(
            1,
            CpuBootInformation {
                uart: shared_device.clone(),
                rand_value: 25,
            },
        );
        write_mailbox_for_core(1, sp, arg0);

        let (arg0, sp) = write_cpu_boot_info(
            2,
            CpuBootInformation {
                uart: shared_device.clone(),
                rand_value: 77,
            },
        );
        write_mailbox_for_core(2, sp, arg0);

        let (arg0, sp) = write_cpu_boot_info(
            3,
            CpuBootInformation {
                uart: shared_device.clone(),
                rand_value: 923,
            },
        );
        write_mailbox_for_core(3, sp, arg0);
    }

    loop {
        {
            let mut lock = shared_device.lock();
            writeln!(lock, "Mailbox Addr: {:x}", MAILBOX_TOP());
        }

        for _ in 0..5_000_000 {
            unsafe {
                asm!("nop");
            }
        }
    }

    /*    let cpuid = cpu::cpuid();

    if cpuid != 0 {
        unsafe {
            mmu::init();
        }
    }

    if cpuid == 0 {
        cpu::disable_irq();
        init_heap();

        {
            let mut lock = UART0.lock();
            lock.set_baud_rate(48_000_000, 115_200);
            lock.set_parity(SerialParity::Even);
            lock.set_data_bits(SerialDataBits::Eight);
            let _ = lock.enable();
        }

        kprintln!("UART initialized...");

        unsafe {
            let start = (&__heap_start as *const _ as *const u8) as usize;
            let end = (&__heap_end as *const _ as *const u8) as usize;

            let _ = kprintln!(
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
        irq.inner_mut().debug();
        irq.enable_irq(153, CpuTarget::Zero);

        irq.register_callback(153, |frame: &mut ExceptionFrame| {
            let char = UART0.lock().read_byte();

            match char {
                Ok(char) => {
                    kprintln!("{}", char as char);

                    if (char == 'j' as u8) {
                        unsafe { jump_to(0x4000_0000) };
                    }

                    if (char == 'd' as u8) {
                        unsafe { cause_data_translation_load(0x9000_0000u64) };
                    }

                    if (char == 'p' as u8) {
                        let mut gic = IRQ.write();
                        kprintln!("Setting new target: {}", unsafe { IRQ_CORE + 1 % 3 });

                        unsafe {
                            match IRQ_CORE {
                                0 => {
                                    gic.set_irq_target_cpu(153, CpuTarget::One);
                                    IRQ_CORE = 1;
                                }
                                1 => {
                                    gic.set_irq_target_cpu(153, CpuTarget::Two);
                                    IRQ_CORE = 2;
                                }
                                _ => {
                                    gic.set_irq_target_cpu(153, CpuTarget::Zero);
                                    IRQ_CORE = 0;
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    kprintln!("NO CHAR FOUND! Err: {:?}", e);
                }
            }
        });

        irq.register_callback(30, |frame: &mut ExceptionFrame| {
            let timer = get_cpu_timer();
            timer.reset();
        });

        drop(irq);
    }

    let _ = kprintln!(">>>> [{}] SP = {:X}", cpuid, cpu::get_sp());

    {
        let mut uart = UART0.lock();
        uart.enable_interrupt();
    }
    kprintln!("[{cpuid}] Enabled UART interrupts!");

    {
        let mut irq = IRQ.read();
        irq.inner().core_init();
    }

    let timer = get_cpu_timer();
    timer.init();
    timer.set_interval(Duration::from_secs(2));
    let _ = timer.enable();

    kprintln!("[{cpuid}] Enabled Timer interrupts!");

    let mut counter = 0;

    if cpuid == 0 {
        unsafe {
            mmu::create_flat_mapping_4g_l3_pages();
            mmu::intentionally_break();
            mmu::init();
        }

        cpu::wake_secondary_cores();
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
    */
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

    let esr = read_sysreg_esr_el1(); // 0x8600000f -->
    let far = read_sysreg_far_el1(); // 0x28a00 -->

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

    // Optional: also dump the PTEs for ELR/FAR using your existing helper(s).
    // dump_pte_for_va(elr as usize);
    // dump_pte_for_va(far as usize);

    // Halt here so you can read the output.
    loop {
        asm!("wfi", options(nomem, nostack, preserves_flags));
    }
}

#[unsafe(no_mangle)]
extern "C" fn el0_sys_write(user_buf: *const u8, len: usize) {
    // Defensive cap to avoid unbounded spam if EL0 passes a huge length.
    let mut remaining = core::cmp::min(len, 4096);

    let mut p = user_buf;
    let mut tmp = [0u8; 128];

    while remaining != 0 {
        let take = core::cmp::min(tmp.len(), remaining);
        unsafe {
            // Straight copy from user VA. If PAN is enabled and SPAN=0, this
            // will fault; in that case, clear PAN around this call (see asm).
            core::ptr::copy_nonoverlapping(p, tmp.as_mut_ptr(), take);
            p = p.add(take);
        }
        remaining -= take;

        // Try UTF-8; if not valid, print hex.
        if let Ok(s) = core::str::from_utf8(&tmp[..take]) {
            kprintln!("{}", s);
        } else {
            for &b in &tmp[..take] {
                unimplemented!()
            }
        }
    }
}

#[unsafe(no_mangle)]
#[unsafe(naked)]
#[rustfmt::skip]
pub unsafe extern "C" fn el0_sync() -> ! {
    // This also handles SVC -> therefore needed to handle EL0 -> EL1 transition

    naked_asm!(
        "mrs x9, ESR_EL1",
        "ubfx x10, x9, #26, #6", // EC
        "cmp x10, #0x15",
        "b.ne el0_handler", // not SVC -> default handler
        // noreturn

        // Check for 0x20 -> uart_write
        "and x11, x9, 0xFFFF",
        "cmp x11, 0x20",
        "b.ne 1f",
        "sub sp, sp, #256",
        "bl el0_sys_write",
        "add sp, sp, #256",
        "eret",

        // Check for 0x10 -> EL0 -> EL1
        "1:",
        "and x11, x9, #0xFFFF",
        "cmp x11, #0x10",
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
        "mov x0, #(1<<9 | 1<<8 | 0<<7 | 1<<6 | 0b0101)", // DAIF = 1111, M=EL1h,
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
    kprintln!("ERR_INVALID");
    loop {}
}

#[unsafe(no_mangle)]
extern "C" fn el0_irq(exception_frame: &mut ExceptionFrame) {
    let (iar, cb, level) = {
        let irq = IRQ.read();
        let iar = irq.read_irq_num();
        let irq_num = (iar & 0x3FF) as usize;

        let core_id = cpu::cpuid();
        kprintln!(
            "[ {} | {}::IRQ @ {} --> {} ]",
            ExceptionLevel::User,
            core_id,
            get_cpu_timer().get_value(),
            irq_num
        );

        (iar, irq.get_callback(irq_num), ExceptionLevel::User)
    };

    if let Some(handler) = cb {
        handler(exception_frame);
    }

    {
        let irq = IRQ.read();
        irq.set_irq_end(iar);
    }
}

static mut IRQ_CORE: u8 = 0;

#[unsafe(no_mangle)]
extern "C" fn el1_irq(exception_frame: &mut ExceptionFrame) {
    let (iar, cb, level) = {
        let irq = IRQ.read();
        let iar = irq.read_irq_num();
        let irq_num = (iar & 0x3FF) as usize;

        let core_id = cpu::cpuid();
        kprintln!(
            "[ {} | {}::IRQ @ {} --> {} ]",
            ExceptionLevel::Kernel,
            core_id,
            get_cpu_timer().get_value(),
            irq_num
        );

        (iar, irq.get_callback(irq_num), ExceptionLevel::Kernel)
        // read guard drops here
    };

    if let Some(handler) = cb {
        handler(exception_frame);
    }

    {
        let irq = IRQ.read();
        irq.set_irq_end(iar);
    }
}

#[unsafe(no_mangle)]
extern "C" fn unhandled_irq() {
    kprintln!("UNHANDLED_IRQ");
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
