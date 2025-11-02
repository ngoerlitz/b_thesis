use crate::drivers::gic400::GIC400;
use crate::hal::irq::InterruptController;
use crate::hal::timer::SystemTimer;
use crate::isr::ExceptionFrame;
use crate::isr::el::ExceptionLevel;
use crate::platform::aarch64::{cpu, get_cpu_timer};
use crate::{IRQ_MANAGER, kprintln};

#[unsafe(no_mangle)]
extern "C" fn exc_irq(exception_frame: &mut ExceptionFrame) {
    let (iar, cb, level) = {
        let irq = IRQ_MANAGER.read();
        let iar = GIC400::read_iar();
        let irq_num = (iar & 0x3FF) as usize;

        let core_id = cpu::cpuid();
        kprintln!(
            "[ {} | {}::IRQ @ {} --> {} ]",
            ExceptionLevel::EL1,
            core_id,
            get_cpu_timer().get_value(),
            irq_num
        );

        (
            iar,
            irq.get_irq_handler(irq_num.into()),
            ExceptionLevel::EL1,
        )
    };

    if let Some(handler) = cb {
        handler(exception_frame);
    }

    GIC400::write_eoir(iar);
}
