#[allow(non_snake_case)]
pub mod CNTP_CTL_EL0 {
    use crate::aarch64_read_write_system_reg;
    use core::arch::asm;

    pub const BIT_IMASK: usize = 1;
    
    pub const ISTATUS: u64 = (1 << 2);
    pub const IMASK: u64 = (1 << 1);
    pub const ENABLE: u64 = (1 << 0);

    pub const DISABLE: u64 = (0 << 0);

    pub struct Register {}

    impl Register {
        aarch64_read_write_system_reg!(u64, "CNTP_CTL_EL0");
    }
}

pub static CNTP_CTL_EL0: CNTP_CTL_EL0::Register = CNTP_CTL_EL0::Register {};
