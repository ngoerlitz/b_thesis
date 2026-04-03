use crate::drivers::pl011::PL011;
use crate::hal::timer::SystemTimerDriver;
use crate::isr::SvcType;
use crate::platform::aarch64::{cpu, get_cpu_timer};
use crate::{kprintln, save_callee_regs, svc_call, uprintln};
use alloc::format;
use alloc::string::String;
use core::arch::{asm, naked_asm};
use core::mem::MaybeUninit;
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

fn switch_to_user(sp: u64, fp: u64, ptr: u64, cpuid: u64) {
    kprintln!("sp: {sp}, fp: {fp}, ptr: {ptr}, cpuid: {cpuid}");

    unsafe {
        asm!(
            "msr DAIFSet, #0b1111",
            "isb",

            "mov x9, sp",                   // Save the current SP

            save_callee_regs!(),            // Save registers x19-x30 (AAPCS)

            "adr x0, 1f",
            "stp x0, x9, [sp, #-16]!",      // Save return addr and SP

            "stp x12, xzr, [sp, #-16]!",    // Save xptr and padding (SP ~ 16 Byte)

            "msr SP_EL0, x10",
            "msr ELR_EL1, x11",

            "msr SPSR_EL1, xzr",

            "mov x0, x13",
            "mov x1, x12",
            "isb",

            "eret",
            "1:",

            in("x10") sp,
            in("x11") fp,
            in("x12") ptr,
            in("x13") cpuid,

            options(preserves_flags),
            clobber_abi("C")
        );
    }
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

    loop {
        switch_to_user(el0_sp, user_fp, x_ptr, cpuid as u64);

        // If you get here, we returned from EL0 via SVC and restored EL1 state.
        kprintln!("[{cpuid}] EL1; x={x}, y={y}, z={z}");
    }
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".user_text")]
pub extern "C" fn user_func(cpu_id: u8, x: *const i32) {
    let sp: u64;
    unsafe { asm!("mov {}, sp", out(reg) sp) }

    uprintln!("Hello world, this is the value of x: {}", unsafe { *x });

    for _ in 0..5_000_000 {
        unsafe { asm!("nop") }
    }

    svc_call!(SvcType::ReturnEl1);
}
