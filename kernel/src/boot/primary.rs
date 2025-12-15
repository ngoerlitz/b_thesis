use crate::actor::env::root::environment::RootEnvironment;
use crate::actor::env::root::service::actor_root_logger_service::ActorRootLoggerService;
use crate::actor::runtime::handler::RuntimeHandler;
use crate::boot::allocator::init_heap;
use crate::boot::global::{ACTOR_ROOT_ENVIRONMENT, IRQ_MANAGER};
use crate::boot::secondary::kernel_secondary;
use crate::boot::{CpuBootInformation, KSTACK_01_TOP, KSTACK_02_TOP, KSTACK_03_TOP, MAILBOX_TOP};
use crate::drivers::pl011::PL011;
use crate::hal::driver::Driver;
use crate::hal::irq::InterruptController;
use crate::hal::irq_driver::{CpuTarget, IrqType};
use crate::platform::aarch64::{cpu, get_cpu_timer};
use crate::test::kernel_func;
use crate::{bsp, kprintln};
use alloc::sync::Arc;
use core::arch::asm;
use core::fmt::{Debug, Formatter};
use core::time::Duration;
use core::{fmt, ptr};
use spin::Mutex;
use zcene_core::actor::{Actor, ActorEnvironmentSpawn, ActorEnvironmentSpawnable};
use zcene_core::future::runtime::FutureRuntime;

#[repr(C)]
#[derive(Copy, Clone)]
struct CpuMailbox {
    sp: u64,
    init_func: u64,
    arg0: u64,
    go: u64,
}

/// Write data to CPU's mailbox (for wake-up configuration)
/// Note that QEMU does **NOT** emulate the `wfe` instruction and instead treats it as a `nop`.
/// This means that any `wfe` is subsequently ignored and all loops building on this requirement
/// are all busy-waited!
pub fn write_mailbox_for_core(core_index: u8, sp: u64, arg0: u64) {
    let base = MAILBOX_TOP();

    let mailbox_addr = base + (core_index as usize) * core::mem::size_of::<CpuMailbox>();

    let mbox_ptr = mailbox_addr as *mut CpuMailbox;

    let mbox_val = CpuMailbox {
        sp,
        init_func: kernel_secondary as *const () as u64,
        arg0,
        go: 1u64,
    };

    unsafe {
        ptr::write_volatile(mbox_ptr, mbox_val);
    }
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

fn write_cpu_boot_info(core_id: u8, info: CpuBootInformation) -> (u64, u64) {
    let stack_top = get_core_stack(core_id) as usize;

    let info_size = core::mem::size_of::<CpuBootInformation>();
    let info_base = stack_top - info_size;

    unsafe {
        core::ptr::write_volatile(info_base as *mut CpuBootInformation, info);
    }

    let tmp = info_base - 16;
    let sp_aligned = tmp & !0xFusize; // clear low 4 bits -> multiple of 16

    (info_base as u64, sp_aligned as u64)
}

pub extern "C" fn kernel_main<A: Actor<RootEnvironment>>(actor: A) {
    init_heap();
    init_mailboxes();

    let x = RootEnvironment::new(
        FutureRuntime::new(RuntimeHandler::default()).unwrap(),
        ActorRootLoggerService::new(PL011::default()),
    );

    unsafe {
        ACTOR_ROOT_ENVIRONMENT
            .get()
            .as_mut()
            .unwrap()
            .write(x.into());
    }

    kprintln!("[INFO] Kernel Initializing");

    {
        let mut irq = RootEnvironment::get().irq_manager().write();
        irq.inner_mut().init();
        irq.enable_irq(IrqType::from(bsp::constants::IRQ_PHYS_TIMER), None);
        irq.set_irq_handler(IrqType::from(bsp::constants::IRQ_PHYS_TIMER), |_| {
            kprintln!("Timer!");

            let timer = get_cpu_timer();
            timer.reset();
        });

        let timer = get_cpu_timer();
        timer.init();
        timer.set_interval(Duration::from_millis(100));
        let _ = timer.enable();

        for _ in 0..100_000 {
            unsafe { asm!("nop") }
        }

        irq.inner_mut().core_init();
        kprintln!("{}", timer);
        kprintln!("{}", irq.inner());
    }

    cpu::enable_irq();

    kernel_func();

    loop {}

    // let _ = RootEnvironment::get().spawn(actor).unwrap();
    //
    // RootEnvironment::get().enter();

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

fn get_core_stack(core_id: u8) -> usize {
    match core_id {
        1 => KSTACK_01_TOP(),
        2 => KSTACK_02_TOP(),
        3 => KSTACK_03_TOP(),
        _ => unimplemented!(),
    }
}
