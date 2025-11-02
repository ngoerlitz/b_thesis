pub(crate) mod cntfrq_el0;
pub(crate) mod cntp_ctl_el0;
pub(crate) mod cntp_cval_el0;
pub(crate) mod cntp_tval_el0;
pub(crate) mod cntpct_el0;
pub(crate) mod esr_el1;
pub(crate) mod far_el1;

#[macro_export]
macro_rules! aarch64_read_write_system_reg {
    ($reg_ty:ty, $reg_name:tt) => {
        #[inline(always)]
        pub fn read(&self) -> $reg_ty {
            let v: $reg_ty;
            unsafe {
                asm!(concat!("mrs {val}, ", $reg_name), val = out(reg) v, options(nostack, preserves_flags));
            }
            v
        }

        #[inline(always)]
        pub fn write(&self, val: $reg_ty) {
            unsafe {
                asm!(concat!("msr ", $reg_name, ", {v}"), v = in(reg) val, options(nostack));
            }
        }
    };
}
