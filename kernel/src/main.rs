#![no_std]
#![no_main]
#![allow(unused, unused_variables)]
#![feature(allocator_api)]
#![feature(format_args_nl)]

extern crate alloc;

mod bsp;
mod drivers;
mod hal;
mod isr;
mod mmu;
mod panic;
mod platform;
mod test;

use crate::drivers::gic400::GIC400;
use crate::drivers::pl011::PL011;
use crate::hal::driver::Driver;
use crate::hal::irq::InterruptController;
use crate::hal::irq_driver::{CpuTarget, IrqType};
use crate::hal::serial::{SerialDataBits, SerialDevice, SerialParity};
use crate::hal::timer::SystemTimer;
use crate::isr::irq_manager::IrqManager;
use crate::mmu::{cause_data_translation_load, jump_to};
use crate::platform::aarch64::{cpu, get_cpu_timer};
use crate::test::kernel_func;
use alloc::collections::btree_map::Entry;
use alloc::sync::Arc;
use core::arch::{asm, naked_asm};
use core::fmt::{Debug, Display, Formatter, Write};
use core::ops::{Deref, DerefMut};
use core::panic::PanicInfo;
use core::time::Duration;
use core::{fmt, ptr, slice};
use linked_list_allocator::LockedHeap;
use spin::{Mutex, RwLock};
use zcene_core::actor::{ActorMessageSender, ActorSystem, ActorSystemReference};
use zcene_core::future::runtime::FutureRuntime;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

unsafe extern "C" {
    static __heap_start: usize;
    static __heap_end: usize;
}

static UART0: Mutex<PL011> = Mutex::new(unsafe { PL011::new(bsp::constants::UART0_BASE) });
static IRQ_MANAGER: RwLock<IrqManager<GIC400, 216>> = RwLock::new(IrqManager::new(GIC400::new()));

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

    kprintln!("Hello World, kernel starting!");

    {
        let mut irq = IRQ_MANAGER.write();
        irq.inner_mut().init();

        irq.enable_irq(
            IrqType::from(bsp::constants::IRQ_PHYS_TIMER),
            CpuTarget::empty(),
        );

        irq.set_irq_handler(IrqType::from(bsp::constants::IRQ_PHYS_TIMER), |_| {
            kprintln!("Timer!");

            let timer = get_cpu_timer();
            timer.reset();
        });

        let timer = get_cpu_timer();
        timer.init();
        timer.set_interval(Duration::from_millis(500));
        let _ = timer.enable();

        irq.inner_mut().core_init();
        kprintln!("{}", timer);
        kprintln!("{}", irq.inner());
    }

    cpu::enable_irq();

    /*
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
    */

    loop {}
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
extern "C" fn unhandled_irq() {
    kprintln!("UNHANDLED_IRQ");
    loop {}
}
