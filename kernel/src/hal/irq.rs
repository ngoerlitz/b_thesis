use crate::hal::irq_driver::{CpuTarget, InterruptGroup, IrqType};
use crate::isr::ExceptionFrame;
use bitflags::bitflags;

pub(crate) trait InterruptController {
    fn enable_irq(&mut self, irq_type: IrqType, cpu: CpuTarget);
    fn disable_irq(&mut self, irq_type: IrqType);
    fn set_irq_target(&mut self, irq_type: IrqType, cpu: CpuTarget);
    fn set_irq_group(&mut self, irq_type: IrqType, group: InterruptGroup);
    fn set_irq_handler(
        &mut self,
        irq_type: IrqType,
        handler: fn(exception_frame: &mut ExceptionFrame),
    );
    fn get_irq_handler(&self, irq_type: IrqType) -> Option<fn(&mut ExceptionFrame)>;
}
