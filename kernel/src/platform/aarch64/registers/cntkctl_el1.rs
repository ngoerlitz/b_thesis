#[allow(non_snake_case)]
pub mod CNTKCTL_EL1 {
    use crate::aarch64_read_write_system_reg;
    use core::arch::asm;

    pub struct Register {}

    impl Register {
        aarch64_read_write_system_reg!(u64, "CNTKCTL_EL1");
    }
}

pub static CNTKCTL_EL1: CNTKCTL_EL1::Register = CNTKCTL_EL1::Register {};