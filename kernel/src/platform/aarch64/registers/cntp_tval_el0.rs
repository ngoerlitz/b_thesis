#[allow(non_snake_case)]
pub mod CNTP_TVAL_EL0 {
    use crate::aarch64_read_write_system_reg;
    use core::arch::asm;

    pub struct Register {}

    impl Register {
        aarch64_read_write_system_reg!(u64, "CNTP_TVAL_EL0");
    }
}

pub static CNTP_TVAL_EL0: CNTP_TVAL_EL0::Register = CNTP_TVAL_EL0::Register {};
