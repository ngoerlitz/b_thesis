use crate::actor::env::root::environment::RootEnvironment;
use crate::boot::global::IRQ_MANAGER;
use crate::drivers::gic400::GIC400;
use crate::hal::irq::InterruptController;
use crate::hal::timer::SystemTimerDriver;
use crate::isr::context::ISRContext;
use crate::isr::el::ExceptionLevel;
use crate::kprintln;
use crate::platform::aarch64::{cpu, get_cpu_timer};

#[unsafe(no_mangle)]
extern "C" fn exc_irq(ctx: &mut ISRContext) {
    let (irq_svc, iar, irq_num, core_id) = super::setup_isr();

    kprintln!(
        "[ {} | {}::IRQ @ {} --> {} ]",
        ExceptionLevel::EL1,
        core_id,
        get_cpu_timer().now().ticks(),
        irq_num
    );

    irq_svc.dispatch(irq_num.into(), ctx);

    GIC400::write_eoir(iar);
}
