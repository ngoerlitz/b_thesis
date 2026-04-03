#[allow(non_snake_case)]
pub mod CNTPCT_EL0 {
    use crate::aarch64_read_write_system_reg;
    use core::arch::asm;

    pub struct Register {}

    impl Register {
        aarch64_read_write_system_reg!(u64, "CNTPCT_EL0");
    }
}

pub static CNTPCT_EL0: CNTPCT_EL0::Register = CNTPCT_EL0::Register {};
