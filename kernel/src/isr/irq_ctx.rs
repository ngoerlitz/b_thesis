use core::fmt;

#[repr(C, align(16))]
#[derive(Debug, Clone, Copy)]
pub struct El0IrqContext {
    pub elr_el1: u64,
    pub spsr_el1: u64,

    pub x: [u64; 31], // x0 - x30
    pub el0_sp: u64,
}