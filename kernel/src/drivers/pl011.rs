use crate::bsp;
use crate::drivers::common::register::RegU32;
use crate::hal::driver::Driver;
use crate::hal::serial::{SerialDataBits, SerialDriver, SerialError, SerialParity};
use core::fmt::Write;
use core::ptr::NonNull;

const FR_RXFE_INDEX: usize = 4;
const FR_TXFF_INDEX: usize = 5;
const FR_RXFF_INDEX: usize = 6;

// Control Register (UARTCR)
const CR_CTSEN: u32 = 1 << 15;
const CR_RTSEN: u32 = 1 << 14;
const CR_OUT2: u32 = 1 << 13;
const CR_OUT1: u32 = 1 << 12;
const CR_RTS: u32 = 1 << 11;
const CR_DTR: u32 = 1 << 10;
const CR_RXE: u32 = 1 << 9;
const CR_TXE: u32 = 1 << 8;
const CR_LBE: u32 = 1 << 7;
const CR_SIRLP: u32 = 1 << 2;
const CR_SIREN: u32 = 1 << 1;
const CR_UARTEN: u32 = 1 << 0;

// Line Control Register (UARTLCR_H)
const LCRH_SPS: u32 = 1 << 7;
const LCRH_WLEN_5: u32 = 0b00 << 5;
const LCRH_WLEN_6: u32 = 0b01 << 5;
const LCRH_WLEN_7: u32 = 0b10 << 5;
const LCRH_WLEN_8: u32 = 0b11 << 5;
const LCRH_FEN: u32 = 1 << 4;
const LCRH_STP2: u32 = 1 << 3;
const LCRH_EPS: u32 = 1 << 2;
const LCRH_PEN: u32 = 1 << 1;
const LCRH_BRK: u32 = 1 << 0;

const IMSC_RXIM: u32 = 1 << 4;
const IMSC_RTIM: u32 = 1 << 6;

const MIS_RXMIS: u32 = 1 << 4;
const MIS_RTMIS: u32 = 1 << 6;

const ICR_RXIC: u32 = 1 << 4;
const ICR_RTIC: u32 = 1 << 6;

#[repr(C)]
pub struct PL011Registers {
    dr: RegU32,
    rsr_ecr: RegU32,
    _reserved_0: [u32; 4],
    fr: RegU32,
    _reserved_1: [u32; 1],
    pr: RegU32,
    ibrd: RegU32,
    fbrd: RegU32,
    lcr_h: RegU32,
    cr: RegU32,
    ifls: RegU32,
    imsc: RegU32,
    ris: RegU32,
    mis: RegU32,
    icr: RegU32,
    dmacr: RegU32,
}

pub struct PL011 {
    base: NonNull<PL011Registers>,
}

unsafe impl Send for PL011 {}

impl PL011 {
    pub const fn new(base: usize) -> Self {
        unsafe {
            Self {
                base: NonNull::new_unchecked(base as *mut PL011Registers),
            }
        }
    }

    fn rx_fifo_empty(&self) -> bool {
        let regs = self.base.as_ptr() as *const PL011Registers;

        unsafe { (*regs).fr.read_bit(FR_RXFE_INDEX) }
    }

    fn is_tx_fifo_full(&self) -> bool {
        let regs = self.base.as_ptr() as *const PL011Registers;

        unsafe { (*regs).fr.read_bit(FR_TXFF_INDEX) }
    }

    fn mis(&self) -> u32 {
        let regs = self.base.as_ptr() as *const PL011Registers;

        unsafe { (*regs).mis.read() }
    }

    pub fn enable_interrupt(&mut self) {
        let regs = self.base.as_ptr();

        unsafe {
            (*regs).icr.write(0x7FF);
            let imsc = (*regs).imsc.read() | IMSC_RXIM | IMSC_RTIM;
            (*regs).imsc.write(imsc);
        }
    }

    pub fn clear_rx_interrupts(&mut self) {
        let regs = self.base.as_ptr();

        unsafe { (*regs).icr.write(ICR_RXIC | ICR_RTIC) }
    }

    fn read_dr(&mut self) -> u8 {
        let regs = self.base.as_ptr();
        unsafe { (*regs).dr.read() as u8 }
    }
}

impl Default for PL011 {
    fn default() -> Self {
        let mut pl011 = unsafe {
            Self {
                base: NonNull::new_unchecked(bsp::constants::UART0_BASE as *mut PL011Registers),
            }
        };

        pl011.set_baud_rate(26, 3);
        pl011.set_parity(SerialParity::None);
        pl011.set_data_bits(SerialDataBits::Eight);

        pl011
    }
}

impl Write for PL011 {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.chars() {
            self.write_byte(c as u8);
        }

        Ok(())
    }
}

impl SerialDriver for PL011 {
    fn set_baud_rate(&mut self, ibrd: u32, fbrd: u32) {
        assert_ne!(ibrd, 0);
        assert_ne!(fbrd, 0);

        let regs = self.base.as_ptr();
        unsafe {
            (*regs).ibrd.write(ibrd);
            (*regs).fbrd.write(fbrd & 0x3F);
        }
    }

    fn set_parity(&mut self, parity: SerialParity) {
        let mut regs = self.base.as_ptr();

        unsafe {
            let mut lcrh = (*regs).lcr_h.read();

            // Clear parity bits
            lcrh &= !(LCRH_PEN | LCRH_EPS | LCRH_SPS);

            match parity {
                SerialParity::None => {}
                SerialParity::Even => {
                    lcrh |= LCRH_PEN | LCRH_EPS;
                }
                SerialParity::Odd => {
                    lcrh |= LCRH_PEN;
                }
            }

            (*regs).lcr_h.write(lcrh);
        }
    }

    fn set_data_bits(&mut self, bits: SerialDataBits) {
        let mut regs = self.base.as_ptr();

        unsafe {
            let mut lcrh = (*regs).lcr_h.read();

            lcrh &= !(LCRH_WLEN_5 | LCRH_WLEN_6 | LCRH_WLEN_7 | LCRH_WLEN_8);

            match bits {
                SerialDataBits::Five => {
                    lcrh |= LCRH_WLEN_5;
                }
                SerialDataBits::Six => {
                    lcrh |= LCRH_WLEN_6;
                }
                SerialDataBits::Seven => {
                    lcrh |= LCRH_WLEN_7;
                }
                SerialDataBits::Eight => {
                    lcrh |= LCRH_WLEN_8;
                }
            };

            (*regs).lcr_h.write(lcrh);
        }
    }

    fn write_byte(&mut self, byte: u8) {
        while self.is_tx_fifo_full() {}

        let regs = self.base.as_ptr();
        unsafe { (*regs).dr.write(byte as u32) };
    }

    fn read_byte(&mut self) -> Result<u8, SerialError> {
        let regs = self.base.as_ptr();

        if self.rx_fifo_empty() {
            return Err(SerialError::TODO);
        }

        let mut char: u8;

        unsafe {
            char = (*regs).dr.read() as u8;
            (*regs).icr.enable_bit(4);
        }

        Ok(char)
    }
}

impl Driver for PL011 {
    const NAME: &'static str = "PL011 - UART Driver";

    fn enable(&mut self) -> Result<(), ()> {
        let regs = self.base.as_ptr();

        unsafe {
            (*regs).cr.zero();
            (*regs).cr.write(CR_TXE | CR_RXE | CR_UARTEN);
        }

        Ok(())
    }

    fn disable(&mut self) {
        let regs = self.base.as_ptr();

        unsafe {
            (*regs).cr.clear_bit(0);
        }
    }
}
