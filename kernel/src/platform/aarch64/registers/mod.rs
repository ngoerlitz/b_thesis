pub mod cntfrq_el0;
pub mod cntp_ctl_el0;
pub mod cntp_cval_el0;
pub mod cntp_tval_el0;
pub mod cntpct_el0;
pub mod esr_el1;
pub mod far_el1;

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

        #[inline(always)]
        pub fn enable_bit(&self, index: usize) {
            let mut val: $reg_ty = self.read();
            val |= 1 << index;
            self.write(val);
        }

        #[inline(always)]
        pub fn clear_bit(&self, index: usize) {
            let mut val: $reg_ty = self.read();
            val &= !(1 << index);
            self.write(val);
        }
    };
}
