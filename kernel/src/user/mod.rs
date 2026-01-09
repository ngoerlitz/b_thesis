use core::arch::asm;

pub mod print;
pub(crate) mod test;

#[macro_export]
macro_rules! uprintln {
    ($($arg:tt)*) => {{
        $crate::user::print::user_print(core::format_args_nl!($($arg)*));
    }}
}

#[macro_export]
macro_rules! svc_call {
    ($svcid: expr) => {
        unsafe { asm!("svc #{imm}", imm = const($svcid as u16), options(nostack)) }
    };
}
