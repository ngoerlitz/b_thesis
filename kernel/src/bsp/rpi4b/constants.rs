pub const GIC400_BASE: usize = 0xFF84_0000;
pub const UART0_BASE: usize = 0xFE20_1000;

pub const IRQ_PHYS_TIMER: usize = 30;
pub const IRQ_UART0: usize = 153;

pub const STACK_SIZE: usize = 0x4000; // 16K
