use crate::kprintln;

#[unsafe(no_mangle)]
extern "C" fn unhandled_irq() {
    kprintln!("UNHANDLED_IRQ");
    loop {}
}
