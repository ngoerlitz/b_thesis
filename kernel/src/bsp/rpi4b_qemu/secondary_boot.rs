use core::arch::asm;
use crate::actor::env::root::environment::RootEnvironment;
use core::fmt::Write;
use core::sync::atomic::Ordering;
use crate::drivers::mmu;
use crate::drivers::pl011::PL011;
use crate::kprintln;

#[unsafe(no_mangle)]
pub(crate) unsafe extern "C" fn _secbt(cpuid: u8, sp: u64) {
    #[cfg(not(feature = "single_core"))]
    {
        unsafe {
            mmu::init_page_tables();
            mmu::init_user_page_tables();
            mmu::enable_mmu_el1();
        }

        kprintln!("Core {cpuid} -> SP: {:#X}", sp);

        RootEnvironment::get().enter();
    }
    loop {}
}