use crate::drivers::pl011::PL011;
use crate::hal::timer::SystemTimerDriver;
use crate::platform::aarch64::{cpu, get_cpu_timer};
use crate::{kprintln, save_gp_regs};
use alloc::format;
use alloc::string::String;
use core::arch::asm;
use core::fmt::Write;
use core::ops::DerefMut;
use core::ptr::addr_of;

pub fn kernel_func() {
    // Runs at EL1
    let timer = get_cpu_timer().now().as_millis();
    let mut x = 5;
    let y = 13;
    let z = String::from("Hello World");

    kprintln!("Address of x: {:x}", (&raw const x) as u64);

    print_kernel_data(&mut x, &y, &z);
}

unsafe extern "C" {
    static __stack_el0_top: usize;
}

fn sp() -> u64 {
    let mut sp: u64 = 0;
    unsafe {
        asm!(
        "mov {}, sp" // x2 = x_ptr
        , out(reg) sp, options(nostack, preserves_flags));
    }

    sp
}

fn user_stack_top() -> usize {
    unsafe { &__stack_el0_top as *const _ as *const u8 as usize }
}

pub fn print_kernel_data(x: &mut i32, y: &i32, z: &String) {
    let cpuid = cpu::cpuid();

    kprintln!(
        "[{cpuid}] \"{z}\", from kernel_func! x = {x}, y = {y} <> [EL0_STACK_TOP = {:x}]",
        user_stack_top()
    );

    for _ in 0..10_000_000 {
        unsafe { asm!("nop") }
    }

    // Drop to EL0 and run user code; resume here after SVC handler restores EL1.
    // unsafe { switch_to_user(user_func as usize as u64, user_stack_top() as u64) };

    let mut el0_sp: u64 = unsafe { &__stack_el0_top as *const _ as u64 };
    el0_sp -= 16384 * cpuid as u64;

    let user_fp: u64 = unsafe { user_func as *const () as u64 };
    let x_ptr: u64 = x as *mut i32 as u64;

    kprintln!(
        "[{cpuid}] CurrentEL: {} | EL0_SP: {:X} | USER_FP: {:X} <> [EL1_SP = {:x}]",
        cpu::current_el(),
        el0_sp,
        user_fp,
        sp()
    );

    // TODO: We need to save CPSR/PSTATE flags here to.

    loop {
        unsafe {
            asm!(
                "msr DAIFSet, #0b1111",
                "isb",

                "mov x9, sp",

                "sub sp, sp, #304",

                save_gp_regs!(),

                "mrs x0, ELR_EL1",
                "mrs x1, SPSR_EL1",
                "stp x0, x1, [sp, #16*15]",

                "adr x0, 1f",
                "str x0,  [sp, #16*16]",
                "str x30, [sp, #16*16 + 8]",
                "str x9,  [sp, #16*17]",
                "str {xptr}, [sp, #16*17 + 8]",

                "str x9, [sp, #16 * 18]",

                "msr SP_EL0, {el0_stack_top}",
                "msr ELR_EL1, {user_func_ptr}",

                "msr SPSR_EL1, xzr",

                "mov x0, {core_id}",
                "isb",

                "eret",
                "1:",

                el0_stack_top = in(reg) el0_sp,
                user_func_ptr = in(reg) user_fp,
                xptr         = in(reg) x_ptr,
                core_id      = in(reg) cpuid,

                out("x9") _,

                // do not claim nostack (we change sp)
                options(preserves_flags),
            );
        }

        {
            unsafe {
                for _ in 0..5_000_000 {
                    asm!("nop");
                }
            }

            // If you get here, we returned from EL0 via SVC and restored EL1 state.
            kprintln!("[{cpuid}] EL1; x={x}, y={y}, z={z}");
        }
    }
}

#[inline(always)]
fn sys_write(buf: *const u8, len: usize) {
    unsafe {
        asm!(
            "svc #0x20",
            in("x0") buf,
            in("x1") len,
            options(nostack, preserves_flags)
        );
    }
}

#[unsafe(link_section = ".user_text")]
struct StackBuf<const N: usize> {
    buf: [u8; N],
    len: usize,
}

#[unsafe(link_section = ".user_text")]
impl<const N: usize> StackBuf<N> {
    #[inline(always)]
    fn new() -> Self {
        Self {
            buf: [0; N],
            len: 0,
        }
    }
    #[inline(always)]
    fn as_ptr_len(&self) -> (*const u8, usize) {
        (self.buf.as_ptr(), self.len)
    }
}

#[unsafe(link_section = ".user_text")]
impl<const N: usize> core::fmt::Write for StackBuf<N> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let b = s.as_bytes();
        if self.len + b.len() > N {
            return Err(core::fmt::Error);
        }
        // safety: bounds checked
        unsafe {
            core::ptr::copy_nonoverlapping(
                b.as_ptr(),
                self.buf.as_mut_ptr().add(self.len),
                b.len(),
            );
        }
        self.len += b.len();
        Ok(())
    }
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".user_text")]
pub fn user_func(cpu_id: u8) {
    let mut sp: u64;
    unsafe { asm!("mov {sp}, sp", sp = out(reg) sp) }

    // Runs at EL0
    let x = 33;
    let y = 25;
    let z = "EL0";

    let mut sbuf: StackBuf<128> = StackBuf::new();
    use core::fmt::Write;
    let _ = write!(
        &mut sbuf,
        "[{}] \"{}\", from user_func! x = {} , y = {} <> [EL0_STACK_TOP = {:#x}]",
        cpu_id, z, x, y, sp
    );

    let (p, n) = sbuf.as_ptr_len();
    sys_write(p, n);
    sys_write(p, n);

    unsafe {
        for _ in 0..5_000_000 {
            asm!("nop");
        }
    }

    // Trigger SVC to return to EL1
    unsafe { asm!("svc #0x10") };

    unreachable!();
    loop {}
}
