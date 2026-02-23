use core::arch::asm;
use crate::actor::env::root::environment::RootEnvironment;
use core::fmt::Write;
use core::sync::atomic::Ordering;
use crate::boot::global::{ACTOR_ROOT_ENVIRONMENT, ROOT_ENVIRONMENT_READY};
use crate::drivers::mmu;
use crate::drivers::pl011::PL011;
use crate::hal::serial::SerialDriver;
use crate::kprintln;

#[unsafe(no_mangle)]
pub(crate) unsafe extern "C" fn _secbt(cpuid: u8) {
    let mpidr: u64;
    asm!("mrs {}, mpidr_el1", out(reg) mpidr, options(nostack, preserves_flags));
    let id = (mpidr & 0xff) as u8;

    let mut p = PL011::default();
    p.write_byte(b'0' + id);

    loop {}


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