#[allow(non_snake_case)]
pub(crate) mod ESR_EL1 {
    use crate::aarch64_read_write_system_reg;
    use core::arch::asm;

    pub struct Register {}

    impl Register {
        aarch64_read_write_system_reg!(u64, "ESR_EL1");
    }
}

pub(crate) static ESR_EL1: ESR_EL1::Register = ESR_EL1::Register {};
