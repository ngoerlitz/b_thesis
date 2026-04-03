use crate::drivers::pl011::PL011;
use alloc::sync::Arc;
use core::fmt;
use core::fmt::{Debug, Formatter};
use spin::Mutex;

pub mod allocator;
pub mod global;
pub mod panic;
pub mod primary;
pub mod secondary;

#[macro_export]
macro_rules! linker_symbols {
    (
        $(
            $name:ident = $linker_sym:ident ;
        )*
    ) => {
        $(
            unsafe extern "C" {
                // We declare the linker symbol as if it were an extern static.
                // We only ever take its address; we never read/write it.
                //
                // Type here: u8 is conventional because it's just "a byte at that address".
                // You could make this configurable, but u8 is correct for “label points here”.
                static $linker_sym: u8;
            }

            #[allow(non_snake_case)]
            pub fn $name() -> usize {
                // SAFETY: We're just taking the address of a linker-defined symbol.
                // This does not dereference it.
                unsafe { core::ptr::addr_of!($linker_sym) as usize}
            }
        )*
    }
}

#[macro_export]
macro_rules! bootstrap_system {
    ($actor: expr) => {
        #[unsafe(no_mangle)]
        fn __kernel_entry() {
            kernel::boot::primary::kernel_main($actor);

            loop {}
        }
    };
}

linker_symbols! {
    MAILBOX_TOP = __mailbox_top;
    KSTACK_01_TOP = __stack_01_el1_top;
    KSTACK_02_TOP = __stack_02_el1_top;
    KSTACK_03_TOP = __stack_03_el1_top;
}

#[repr(C)]
pub struct CpuBootInformation {
    pub uart: Arc<Mutex<PL011>>,
    pub rand_value: u64,
}

impl Debug for CpuBootInformation {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("CpuBootInformation")
            .field("rand_value", &self.rand_value)
            .finish()
    }
}
