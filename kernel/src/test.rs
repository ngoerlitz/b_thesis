use crate::drivers::pl011::PL011;
use crate::platform::aarch64::cpu;
use crate::{UART0, UartSink};
use alloc::string::String;
use core::arch::asm;
use core::fmt::Write;
use core::ops::DerefMut;
// --- Your app code ---

pub fn kernel_func() {
    // Runs at EL1
    let mut x = 15;
    let y = 13;
    let z = String::from("Hello World");

    print_kernel_data(&mut x, &y, &z);
}

unsafe extern "C" {
    static __stack_el0_top: usize;
}

fn user_stack_top() -> usize {
    unsafe { &__stack_el0_top as *const _ as *const u8 as usize }
}

pub fn print_kernel_data(x: &mut i32, y: &i32, z: &String) {
    let mut lock = UART0.lock();

    let _ = writeln!(
        lock.deref_mut(),
        "\"{z}\", from kernel_func! x = {x}, y = {y} <> [EL0_STACK_TOP = {:x}]",
        user_stack_top()
    );

    for _ in 0..10_000_000 {
        unsafe { asm!("nop") }
    }

    // Drop to EL0 and run user code; resume here after SVC handler restores EL1.
    // unsafe { switch_to_user(user_func as usize as u64, user_stack_top() as u64) };

    let el0_sp: u64 = unsafe { &__stack_el0_top as *const _ as u64 };
    let user_fp: u64 = unsafe { user_func as *const () as u64 };
    let x_ptr: u64 = x as *mut i32 as u64;

    let _ = writeln!(lock.deref_mut(), "CurrentEL: {}", cpu::current_el());

    drop(lock);

    // TODO: We need to save CPSR/PSTATE flags here to.

    loop {
        unsafe {
            asm!(
            "sub sp, sp, #288",

            // store general purpose registers
            "stp x0, x1, [sp, #16 * 0]",
            "stp x2, x3, [sp, #16 * 1]",
            "stp x4, x5, [sp, #16 * 2]",
            "stp x6, x7, [sp, #16 * 3]",
            "stp x8, x9, [sp, #16 * 4]",
            "stp x10, x11, [sp, #16 * 5]",
            "stp x12, x13, [sp, #16 * 6]",
            "stp x14, x15, [sp, #16 * 7]",
            "stp x16, x17, [sp, #16 * 8]",
            "stp x18, x19, [sp, #16 * 9]",
            "stp x20, x21, [sp, #16 * 10]",
            "stp x22, x23, [sp, #16 * 11]",
            "stp x24, x25, [sp, #16 * 12]",
            "stp x26, x27, [sp, #16 * 13]",
            "stp x28, x29, [sp, #16 * 14]",


            // ELR/SPSR pair
            "mrs x0, ELR_EL1",
            "mrs x1, SPSR_EL1",
            "stp x0, x1, [sp, #16 * 15]",    // offset 240

            // LR (x30) and 8-byte padding to keep 16B alignment
            "adr x0, 1f",
            "str x0, [sp, #16 * 16]",        // offset 256

            // Store &mut x
            "str {xptr}, [sp, #16 * 17]",

            // We've stored all registers, now lets go to EL0
            "msr SP_EL0, {el0_stack_top}",
            "msr ELR_EL1, {user_func_ptr}",
            "mov  x0, #(1<<9 | 1<<8 | 1<<7)",   // D|A|I masks
            "msr  SPSR_EL1, xzr",                // EL0t + DAIF masked

            "isb",
            "eret",

            "1:",

            el0_stack_top = in(reg) el0_sp,
            user_func_ptr = in(reg) user_fp,
            xptr = in(reg) x_ptr,
            );
        }

        // If you get here, we returned from EL0 via SVC and restored EL1 state.
        let _ = writeln!(UartSink, "Back in kernel; x={x}, y={y}, z={z}");

        for _ in 0..500_000 {
            unsafe { asm!("nop") }
        }
    }
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".user_text")]
pub fn user_func() {
    let mut sp: u64;
    unsafe { asm!("mov {sp}, sp", sp = out(reg) sp) }

    // Runs at EL0
    let x = 33;
    let y = 25;
    let z = "Adios World";

    // This really isn't great... Directly writing to MMIO from user-mode is kinda uncool :D
    let mut uart = unsafe { PL011::new(0xFE201000) };

    let _ = writeln!(
        uart,
        "\"{z}\", from user_func! x = {x}, y = {y} <> [EL0_STACK_TOP = {:x}]",
        sp
    );

    for _ in 0..500_000 {
        unsafe { asm!("nop") }
    }

    // Trigger SVC to return to EL1
    unsafe { asm!("svc #0x10") };

    unreachable!();
    loop {}
}
