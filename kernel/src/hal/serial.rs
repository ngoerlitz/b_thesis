use crate::hal::driver::Driver;
use core::fmt;
use core::fmt::{Display, Formatter, Write};

#[derive(Debug)]
pub enum SerialError {
    TODO,
    TransmitBufferFull,
}

#[derive(Debug)]
pub enum SerialParity {
    None,
    Even,
    Odd,
}

#[derive(Debug)]
pub enum SerialDataBits {
    Five,
    Six,
    Seven,
    Eight,
}

pub trait SerialDriver: Driver + Write {
    /// Sets the baud rate for the target `SerialDevice`
    fn set_baud_rate(&mut self, ibrd: u32, fbrd: u32);

    /// Sets the parity bits for the target `SerialDevice`
    fn set_parity(&mut self, parity: SerialParity);

    /// Sets the data bits for the target `SerialDevice`
    fn set_data_bits(&mut self, bits: SerialDataBits);

    /// This function blocks / busy-waits until the device is available to write to.
    fn write_byte(&mut self, byte: u8);

    fn read_byte(&mut self) -> Result<u8, SerialError>;
}
