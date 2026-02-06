use core::fmt;
use crate::hal::serial::{SerialDriver, SerialError};
use crate::platform::aarch64;
use crate::platform::aarch64::cpu;
use core::fmt::Write;
use spin::Mutex;

pub struct ActorRootLoggerService<T: SerialDriver> {
    driver: Mutex<T>,
}

impl<T: SerialDriver> ActorRootLoggerService<T> {
    pub fn new(driver: T) -> Self {
        Self {
            driver: Mutex::new(driver),
        }
    }

    pub fn write<S>(&self, string: S) -> Result<(), core::fmt::Error>
    where
        S: AsRef<str>,
    {
        aarch64::irq::run_masked(|| self.driver.lock().write_str(string.as_ref()))
    }

    pub fn write_fmt_locked(&self, args: fmt::Arguments<'_>) -> fmt::Result {
        aarch64::irq::run_masked(|| {
            let mut drv = self.driver.lock();
            drv.write_fmt(args)
        })
    }

    pub fn writer<'a>(&'a self) -> impl Write + 'a {
        ActorRootWriter { service: self }
    }

    pub fn read_char(&self) -> Option<u8> {
        aarch64::irq::run_masked(|| self.driver.lock().read_byte().ok())
    }
}

pub struct ActorRootWriter<'a, T: SerialDriver> {
    service: &'a ActorRootLoggerService<T>,
}

impl<T: SerialDriver> Write for ActorRootWriter<'_, T> {
    fn write_str(&mut self, s: &str) -> Result<(), core::fmt::Error> {
        self.service.write(s)
    }
}
