use core::arch::asm;
use crate::actor::env::root::environment::RootEnvironment;
use core::fmt::Write;
use crate::boot::global::ACTOR_ROOT_ENVIRONMENT;
use crate::drivers::mmu;
use crate::drivers::pl011::PL011;
use crate::kprintln;

#[unsafe(no_mangle)]
pub(crate) unsafe extern "C" fn _secbt(cpuid: u8) {
    #[cfg(not(feature = "single_core"))]
    {
        unsafe {
            mmu::enable_mmu_el1();
        }

        RootEnvironment::get().enter();
    }

    loop {}
    

    // let mut uart = PL011::default();
    //
    // loop {
    //     write!(uart, "Hello, I am on a different core! [{}]\n", cpuid);
    //
    //     for _ in 0..100_000 {
    //         unsafe { asm!("nop"); }
    //     }
    // }

    loop {}
}