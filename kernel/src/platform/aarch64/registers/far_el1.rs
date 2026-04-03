#[allow(non_snake_case)]
pub mod FAR_EL1 {
    use crate::aarch64_read_write_system_reg;
    use core::arch::asm;

    pub struct Register {}

    impl Register {
        aarch64_read_write_system_reg!(u64, "FAR_EL1");
    }
}

pub static FAR_EL1: FAR_EL1::Register = FAR_EL1::Register {};
