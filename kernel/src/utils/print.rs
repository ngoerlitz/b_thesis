use crate::actor::env::root::environment::RootEnvironment;
use crate::platform::aarch64::cpu;

#[macro_export]
macro_rules! kprint {
    ($($arg:tt)*) => {
        $crate::utils::print::kprint(format_args!($($arg)*));
    }
}

#[macro_export]
macro_rules! kprintln {
    ($($arg:tt)*) => {{
        $crate::utils::print::kprint(core::format_args_nl!($($arg)*));
    }}
}

pub fn kprint(args: core::fmt::Arguments) {
    let mut logger = RootEnvironment::get().logger().writer();
    let _ = core::fmt::write(&mut logger, args);
}
