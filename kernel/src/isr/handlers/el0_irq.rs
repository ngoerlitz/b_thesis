use core::arch::asm;
use crate::boot::global::IRQ_MANAGER;
use crate::drivers::gic400::GIC400;
use crate::hal::irq::InterruptController;
use crate::hal::timer::SystemTimerDriver;
use crate::isr::context::{EL1Context, ISRContext};
use crate::isr::el::ExceptionLevel;
use crate::{bsp, log_dbg, kprintln};
use crate::platform::aarch64::{cpu, get_cpu_timer};
use core::ops::Deref;
use crate::actor::env::user::executor_event::{IrqExecutorType, IrqType, UserExecutorEvent};
use crate::isr::irq_ctx::El0IrqContext;

#[unsafe(no_mangle)]
extern "C" fn el0_irq(ctx: *const El0IrqContext, ctx_el1: *mut EL1Context) {
    let (irq_svc, iar, irq_num, core_id) = super::setup_isr();

    log_dbg!(
        "[ {} | {}::IRQ @ {} --> {} ]",
        ExceptionLevel::EL0,
        core_id,
        get_cpu_timer().now().ticks(),
        irq_num
    );

    let irq_type = match (irq_num as usize) {
        bsp::constants::IRQ_PHYS_TIMER  => IrqType::Preemption,
        bsp::constants::IRQ_UART0 => IrqType::UartRx,
        _ => IrqType::Unknown,
    };

    unsafe {
        *(*ctx_el1).event = Some(UserExecutorEvent::Irq(IrqExecutorType {
            ctx: (*ctx).clone(),
            irq_type,
            iar
        }))
    }

    unsafe {
        asm!(
            // Restore callee-saved regs from EL1Context
            "ldp x29, x30, [x1, #112]",
            "ldp x27, x28, [x1, #96]",
            "ldp x25, x26, [x1, #80]",
            "ldp x23, x24, [x1, #64]",
            "ldp x21, x22, [x1, #48]",
            "ldp x19, x20, [x1, #32]",

            // Load resume PC and SP
            "ldp x0, x1, [x1, #0]",
            "msr ELR_EL1, x0",
            "mov sp, x1",

            "mov x2, #( (1<<9) | (1<<8) | (0<<7) | (1<<6) | 0b0101 )",
            "msr SPSR_EL1, x2",

            "isb",
            "eret",

            in("x1") (ctx_el1 as u64),

            options(noreturn),
        );
    }
}
