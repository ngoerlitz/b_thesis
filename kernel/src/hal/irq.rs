use crate::exc_vec::ExceptionFrame;
use bitflags::bitflags;

bitflags! {
    pub(crate) struct CpuTarget: u8 {
        const Zero  = 0b00000001;
        const One   = 0b00000010;
        const Two   = 0b00000100;
        const Three = 0b00001000;
        const Four  = 0b00010000;
        const Five  = 0b00100000;
        const Six   = 0b01000000;
        const Seven = 0b10000000;
    }
}

pub(crate) enum InterruptGroup {
    Zero,
    One,
}

pub(crate) trait InterruptController {
    fn enable_irq(&mut self, irq_num: u32, cpu: CpuTarget);
    fn disable_irq(&mut self, irq_num: u32);
    fn set_irq_target_cpu(&mut self, irq_num: u32, cpu: CpuTarget);
    fn set_irq_target_group(&mut self, irq_num: u32, interrupt_group: InterruptGroup);
    fn read_irq_num(&self) -> u32;
    fn set_irq_end(&self, value: u32);
    fn register_callback(
        &mut self,
        irq_num: u32,
        handler: fn(exception_frame: &mut ExceptionFrame),
    );
}
