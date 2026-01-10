use crate::kprintln;

#[cfg(feature = "log_debug")]
#[macro_export]
macro_rules! log_dbg {
    ($($arg:tt)*) => {{
        kprintln!("[DEBUG {}:{}] {}", file!(), line!(), format_args!($($arg)*));
    }};
}

#[cfg(feature = "log_debug")]
#[macro_export]
macro_rules! log_dbg_naked {
    ($($arg:tt)*) => {{
        kprintln!("[DEBUG] --> {}", format_args!($($arg)*));
    }};
}

#[cfg(not(feature = "log_debug"))]
#[macro_export]
macro_rules! log_dbg {
    ($($arg:tt)*) => {{
    }};
}

#[cfg(not(feature = "log_debug"))]
#[macro_export]
macro_rules! log_dbg_naked {
    ($($arg:tt)*) => {{
    }};
}