use crate::hal::irq_driver::{CpuTarget, InterruptGroup, IrqType};
use crate::isr::context::ISRContext;
use crate::isr::el::ExceptionLevel;
use bitflags::bitflags;

pub trait InterruptController {
    fn enable_irq(&mut self, irq_type: IrqType, cpu: Option<CpuTarget>);
    fn disable_irq(&mut self, irq_type: IrqType);
    fn set_irq_target(&mut self, irq_type: IrqType, cpu: CpuTarget);
    fn set_irq_group(&mut self, irq_type: IrqType, group: InterruptGroup);
    fn set_irq_handler(&mut self, irq_type: IrqType, handler: fn(exception_frame: &mut ISRContext));
    fn get_irq_handler(&self, irq_type: IrqType) -> Option<fn(&mut ISRContext)>;
    fn dispatch(&self, irq_type: IrqType, el: &mut ISRContext);
}
