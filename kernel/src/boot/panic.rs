use crate::bsp;
use crate::drivers::pl011::PL011;
use crate::hal::driver::Driver;
use crate::platform::aarch64::cpu;
use core::fmt::Write;
use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    cpu::disable_irq();
    let core_id = cpu::cpuid();

    // Here, we re-initialize the PL011, since the lock may still be held by a different
    // core. We have to accept the fact that there may be some weird things going on when writing
    // concurrently - ideally, we'd use UART1-3 for this, but that's not emulated by QEMU iirc...
    // Otherwise, we'd not get ANY output, which is arguably worse :D

    let mut uart = unsafe { PL011::new(bsp::constants::UART0_BASE) };
    let _ = uart.enable();

    {
        let _ = writeln!(uart, "[Core: {}] Kernel Panic!\n{}", core_id, info);
    }
    loop {}
}
