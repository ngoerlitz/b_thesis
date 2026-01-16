#[repr(C, align(16))]
#[derive(Copy, Clone, Debug)]
pub struct SyscallContext {
    xn: [u64; 10], // x19 - x28
    elr_el1: u64,
    spsr_el1: u64,
    x30: u64,
    x29: u64
}