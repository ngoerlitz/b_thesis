use crate::hal::serial::SerialDriver;
use crate::platform::aarch64::cpu;

#[macro_export]
macro_rules! kprint {
    ($($arg:tt)*) => {
        $crate::services::logger::kprint(format_args!($($arg)*));
    }
}
#[macro_export]
macro_rules! kprintln {
    ($($arg:tt)*) => {{
        $crate::services::logger::kprint(core::format_args_nl!($($arg)*));
    }}
}

pub struct LoggerService<T: SerialDriver> {
    driver: T,
}

impl<T: SerialDriver> LoggerService<T> {
    pub fn new(driver: T) -> Self {
        Self { driver }
    }

    pub fn init(&mut self) {
        self.driver.enable();

        kprintln!("[Logger] Loaded Driver: \"{}\"", T::NAME);
    }

    pub fn inner(&self) -> &T {
        &self.driver
    }

    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.driver
    }
}

pub fn kprint(args: core::fmt::Arguments) {
    use core::fmt::Write;

    cpu::with_irq_masked(|| {
        let mut guard = crate::boot::global::UART0.lock();
        let _ = core::fmt::write(&mut *guard, args);
    });
}
