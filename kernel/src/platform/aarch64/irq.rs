use crate::platform::aarch64::cpu;

pub fn run_masked<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    let daif = cpu::read_daif();
    cpu::disable_irq();

    let val: R = f();

    cpu::write_daif(daif);
    val
}
