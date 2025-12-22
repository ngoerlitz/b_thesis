use crate::actor::env::root::environment::RootEnvironment;
use crate::drivers::gic400::GIC400;
use crate::kprintln;
use crate::platform::aarch64::cpu;
use crate::services::irq_manager::IrqManagerService;
use spin::RwLockReadGuard;

mod el0_irq;
mod el0_sync;
pub mod exc_irq;
pub mod exc_serror;
mod unhandled;

#[inline]
fn setup_isr() -> (
    RwLockReadGuard<'static, IrqManagerService<GIC400, 216>>,
    u32,
    u32,
    u8,
) {
    let irq_manager = RootEnvironment::get().irq_manager().read();
    let (mut iar, mut irq_num) = GIC400::irq_info();
    let core_id = cpu::cpuid();

    (irq_manager, iar, irq_num, core_id)
}
