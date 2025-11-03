#[allow(non_snake_case)]
pub mod CNTFRQ_EL0 {
    use crate::aarch64_read_write_system_reg;
    use core::arch::asm;

    pub struct Register {}

    impl Register {
        aarch64_read_write_system_reg!(u64, "CNTFRQ_EL0");
    }
}

pub static CNTFRQ_EL0: CNTFRQ_EL0::Register = CNTFRQ_EL0::Register {};
