use crate::hal::driver::Driver;
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

#[derive(Debug, Copy, Clone)]
pub enum IrqType {
    Sgi(u8),  // 0..=15
    Ppi(u8),  // 16..=31
    Spi(u16), // 32..=1019
}

impl From<usize> for IrqType {
    fn from(irq_type: usize) -> Self {
        if irq_type <= 15 {
            return Self::Sgi(irq_type as u8);
        }

        if irq_type <= 31 {
            return Self::Ppi(irq_type as u8);
        }

        Self::Spi(irq_type as u16)
    }
}

impl From<IrqType> for usize {
    fn from(value: IrqType) -> Self {
        match value {
            IrqType::Sgi(irq_type) => irq_type as usize,
            IrqType::Ppi(irq_type) => irq_type as usize,
            IrqType::Spi(irq_type) => irq_type as usize,
        }
    }
}

pub(crate) enum InterruptGroup {
    Zero,
    One,
}

pub trait IrqDriver: Driver {
    fn enable_irq(&mut self, irq_type: IrqType, cpu: CpuTarget);
    fn disable_irq(&mut self, irq_type: IrqType);
    fn set_irq_target(&mut self, irq_type: IrqType, cpu: CpuTarget);
    fn set_irq_group(&mut self, irq_type: IrqType, group: InterruptGroup);
}
