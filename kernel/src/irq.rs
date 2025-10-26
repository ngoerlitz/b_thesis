use crate::drivers::gic400::GIC400;
use crate::exc_vec::ExceptionFrame;
use crate::hal::driver::Driver;
use crate::hal::irq::{CpuTarget, InterruptController, InterruptGroup};
use crate::hal::timer::SystemTimer;
use crate::platform::aarch64::{cpu, get_cpu_timer};
use crate::{UART0, kprintln};
use core::fmt::{Display, Formatter, Write};
use core::ops::DerefMut;

const N: usize = 216;

pub(crate) struct IRQHandler {
    gic: GIC400,
    callback: [Option<fn(&mut ExceptionFrame)>; N],
}

pub(crate) enum ExceptionLevel {
    Firmware,
    Virtualization,
    Kernel,
    User,
}

impl Display for ExceptionLevel {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            ExceptionLevel::Firmware => f.write_str("Firmware [EL3]"),
            ExceptionLevel::Virtualization => f.write_str("Virtualization [EL2]"),
            ExceptionLevel::Kernel => f.write_str("Kernel [EL1]"),
            ExceptionLevel::User => f.write_str("User [EL0]"),
        }
    }
}

impl IRQHandler {
    pub(crate) const fn new(gic: GIC400) -> Self {
        Self {
            gic,
            callback: [None; N],
        }
    }

    pub(crate) fn handle(&self, exception_frame: &mut ExceptionFrame, level: ExceptionLevel) {
        let iar = self.read_irq_num();
        let irq_num = iar & 0x3FF;

        {
            let core_id = cpu::cpuid();
            kprintln!(
                "[ {} | {}::IRQ @ {} --> {} ]",
                level,
                core_id,
                get_cpu_timer().get_value(),
                irq_num
            );
        }

        if let Some(handler) = self.callback[irq_num as usize] {
            handler(exception_frame);
        }

        self.set_irq_end(iar);
    }

    pub(crate) fn get_callback(&self, irq_num: usize) -> Option<fn(&mut ExceptionFrame)> {
        self.callback[irq_num]
    }

    pub(crate) fn inner(&self) -> &GIC400 {
        &self.gic
    }

    pub(crate) fn inner_mut(&mut self) -> &mut GIC400 {
        &mut self.gic
    }
}

impl InterruptController for IRQHandler {
    fn enable_irq(&mut self, irq_num: u32, cpu: CpuTarget) {
        self.gic.enable_irq(irq_num, cpu);
    }

    fn disable_irq(&mut self, irq_num: u32) {
        self.gic.disable_irq(irq_num);
    }

    fn set_irq_target_cpu(&mut self, irq_num: u32, cpu: CpuTarget) {
        self.gic.set_irq_target_cpu(irq_num, cpu);
    }

    fn set_irq_target_group(&mut self, irq_num: u32, interrupt_group: InterruptGroup) {
        self.gic.set_irq_target_group(irq_num, interrupt_group);
    }

    fn read_irq_num(&self) -> u32 {
        self.gic.read_iar()
    }

    fn set_irq_end(&self, value: u32) {
        self.gic.write_eoir(value);
    }

    fn register_callback(&mut self, irq_num: u32, handler: fn(&mut ExceptionFrame)) {
        self.callback[irq_num as usize] = Some(handler);
    }
}
