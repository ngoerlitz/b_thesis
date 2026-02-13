use crate::actor::env::root::environment::RootEnvironment;
use crate::actor::env::root::service::actor_root_logger_service::ActorRootLoggerService;
use crate::actor::runtime::handler::RuntimeHandler;
use crate::boot::global::{ACTOR_ROOT_ENVIRONMENT, IRQ_MANAGER};
use crate::boot::secondary::kernel_secondary;
use crate::boot::{
    CpuBootInformation, KSTACK_01_TOP, KSTACK_02_TOP, KSTACK_03_TOP, MAILBOX_TOP, allocator,
};
use crate::bsp::constants::IRQ_UART0;
use crate::drivers::pl011::PL011;
use crate::hal::driver::Driver;
use crate::hal::irq::InterruptController;
use crate::hal::irq_driver::{CpuTarget, IrqType};
use crate::hal::serial::SerialDriver;
use crate::hal::timer::SystemTimerDriver;
use crate::platform::aarch64::{cpu, get_cpu_timer};
use crate::{bsp, drivers, kprintln, linker_symbols, log_dbg, test, user};
use alloc::sync::Arc;
use core::arch::asm;
use core::fmt::{Debug, Display, Formatter};
use core::time::Duration;
use core::{fmt, ptr};
use spin::Mutex;
use zcene_core::actor::{Actor, ActorEnvironmentSpawn, ActorEnvironmentSpawnable};
use zcene_core::future::runtime::FutureRuntime;

pub static UART0: spin::mutex::Mutex<PL011> =
    spin::mutex::Mutex::new(unsafe { PL011::new(bsp::constants::UART0_BASE) });

linker_symbols! {
    EL1_STACK_TOP = __kstack_end;
    EL1_STACK_SIZE = KSTACK_SIZE;
}

unsafe extern "C" {
    fn _el3();
}

pub extern "C" fn kernel_main<A: Actor<RootEnvironment>>(actor: A) {
    unsafe {
        drivers::mmu::init_page_tables();
        drivers::mmu::init_user_page_tables();
        drivers::mmu::enable_mmu_el1();
    }

    allocator::init_heap();

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

    #[cfg(feature = "test")]
    test::test_all();

    // Bring up the other cores
    unsafe {
        // 0xD8
        // 0xE0
        // 0xE8
        // 0xF0
        ((0xE0) as *mut u64).write_volatile(_el3 as usize as u64);
        ((0xE8) as *mut u64).write_volatile(_el3 as usize as u64);
        ((0xF0) as *mut u64).write_volatile(_el3 as usize as u64);
    }

    kprintln!("[INFO] Kernel Initializing.");
    kprintln!("STACKS");
    for i in 0..4 {
        kprintln!("{i} ]{:#0X} - {:#0X}]", EL1_STACK_TOP() - (EL1_STACK_SIZE() * (i + 1)), EL1_STACK_TOP() - (EL1_STACK_SIZE() * i));
    }

    {
        let mut irq = RootEnvironment::get().irq_manager().write();
        irq.inner_mut().init();
        irq.enable_irq(IrqType::from(bsp::constants::IRQ_PHYS_TIMER), None);
        irq.set_irq_handler(IrqType::from(bsp::constants::IRQ_PHYS_TIMER), |_| {
            kprintln!("Timer!");

            let timer = get_cpu_timer();
            timer.reset();
        });

        {
            UART0.lock().enable().unwrap();
            UART0.lock().enable_interrupt();
        }

        irq.enable_irq(IrqType::from(IRQ_UART0), Some(CpuTarget::Zero));
        irq.set_irq_handler(IrqType::from(bsp::constants::IRQ_UART0), |_| {
            let char = UART0.lock().read_byte();

            match char {
                Ok(c) => {
                    kprintln!("Received: {}", c as char);
                }
                Err(_) => kprintln!("Error reading"),
            }
        });

        let timer = get_cpu_timer();
        timer.init();

        irq.inner_mut().core_init();
        kprintln!("{}", timer);
        kprintln!("{}", irq.inner());
    }

    let _ = RootEnvironment::get().spawn(actor).unwrap();
    RootEnvironment::get().enter();

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
