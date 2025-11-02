use crate::drivers::gic400::GIC400;
use crate::hal::driver::Driver;
use crate::hal::irq::InterruptController;
use crate::hal::irq_driver::{CpuTarget, InterruptGroup, IrqDriver, IrqType};
use crate::isr::ExceptionFrame;
use core::fmt::{Display, Formatter};

pub struct IrqManager<T: IrqDriver, const N: usize> {
    driver: T,
    callback: [Option<fn(&mut ExceptionFrame)>; N],
}

impl<T: IrqDriver, const N: usize> IrqManager<T, N> {
    pub const fn new(driver: T) -> Self {
        Self {
            driver,
            callback: [None; N],
        }
    }

    pub fn inner(&self) -> &T {
        &self.driver
    }

    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.driver
    }

    pub fn enable(&mut self) -> Result<(), ()> {
        self.driver.enable()
    }

    pub fn disable(&mut self) {
        self.driver.disable();
    }
}

impl<T: IrqDriver, const N: usize> Display for IrqManager<T, N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "IRQ Manager [driver: {}]", T::NAME)
    }
}

impl<T: IrqDriver, const N: usize> InterruptController for IrqManager<T, N> {
    fn enable_irq(&mut self, irq_type: IrqType, cpu: CpuTarget) {
        self.driver.enable_irq(irq_type, cpu);
    }

    fn disable_irq(&mut self, irq_type: IrqType) {
        self.driver.disable_irq(irq_type);
    }

    fn set_irq_target(&mut self, irq_type: IrqType, cpu: CpuTarget) {
        self.driver.set_irq_target(irq_type, cpu);
    }

    fn set_irq_group(&mut self, irq_type: IrqType, group: InterruptGroup) {
        self.driver.set_irq_group(irq_type, group);
    }

    fn set_irq_handler(&mut self, irq_type: IrqType, handler: fn(&mut ExceptionFrame)) {
        let idx: usize = irq_type.into();
        assert!(
            idx < self.callback.len(),
            "IRQ Index out of range for Callback-Array"
        );

        self.callback[idx] = Some(handler);
    }

    fn get_irq_handler(&self, irq_type: IrqType) -> Option<fn(&mut ExceptionFrame)> {
        let idx: usize = irq_type.into();
        self.callback[idx]
    }
}
