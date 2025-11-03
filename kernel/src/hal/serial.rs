use crate::hal::driver::Driver;
use core::fmt::Write;

#[derive(Debug)]
pub enum SerialError {
    TODO,
    TransmitBufferFull,
}

pub enum SerialParity {
    None,
    Even,
    Odd,
}

pub enum SerialDataBits {
    Five,
    Six,
    Seven,
    Eight,
}

pub trait SerialDriver: Driver + Write {
    /// Sets the baud rate for the target `SerialDevice`
    fn set_baud_rate(&mut self, uart_clk_hz: u32, baud: u32);

    /// Sets the parity bits for the target `SerialDevice`
    fn set_parity(&mut self, parity: SerialParity);

    /// Sets the data bits for the target `SerialDevice`
    fn set_data_bits(&mut self, bits: SerialDataBits);

    /// This function blocks / busy-waits until the device is available to write to. Unlike
    /// `try_write_byte`, no `Result` is returned as it is assumed the write operation succeeds
    /// when waiting long enough.
    fn write_byte(&mut self, byte: u8);

    /// This function tries to write to the device immediately, returning a `SerialError` if this
    /// is currently not possible (e.g. due to the TX-FIFO being full).
    fn try_write_byte(&mut self, byte: u8) -> Result<(), SerialError>;

    /// Attempts to write as much of the string's bytes as possible using `try_write_byte`. As
    /// soon as an Error is encountered, this function propagates the error whilst including the
    /// current index of the byte, during which the `try_write_byte` failed.
    fn try_write_string(&mut self, string: &str) -> Result<(), (usize, SerialError)> {
        for (idx, byte) in string.bytes().enumerate() {
            self.try_write_byte(byte).map_err(|e| (idx, e))?;
        }

        Ok(())
    }

    fn read_byte(&mut self) -> Result<u8, SerialError>;
}
