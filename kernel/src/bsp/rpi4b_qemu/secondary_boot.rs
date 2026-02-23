use core::arch::asm;
use crate::actor::env::root::environment::RootEnvironment;
use core::fmt::Write;
use core::sync::atomic::Ordering;
use crate::boot::global::{ACTOR_ROOT_ENVIRONMENT, ROOT_ENVIRONMENT_READY};
use crate::drivers::mmu;
use crate::drivers::pl011::PL011;
use crate::kprintln;

#[unsafe(no_mangle)]
pub(crate) unsafe extern "C" fn _secbt(cpuid: u8) {
    #[cfg(not(feature = "single_core"))]
    {
        unsafe {
            mmu::init_page_tables();
            mmu::init_user_page_tables();
            mmu::enable_mmu_el1();
        }

        while ROOT_ENVIRONMENT_READY.load(Ordering::Acquire) == 0 {
            core::arch::asm!("wfe", options(nostack, nomem, preserves_flags));
        }

        RootEnvironment::get().enter();
    }
    loop {}
}