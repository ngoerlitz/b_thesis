use crate::{UART0, UartSink};
use alloc::string::String;
use core::arch::asm;
use core::fmt::Write;
use core::ops::DerefMut;
// --- Your app code ---

pub fn kernel_func() {
    // Runs at EL1
    let x = 15;
    let y = 13;
    let z = String::from("Hello World");

    print_kernel_data(x, &y, &z);
}

unsafe extern "C" {
    static __stack_el0_top: usize;
}

fn user_stack_top() -> usize {
    unsafe { &__stack_el0_top as *const _ as *const u8 as usize }
}

pub fn print_kernel_data(x: i32, y: &i32, z: &String) {
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

    let _ = writeln!(
        lock.deref_mut(),
        "CurrentEL: {}",
        crate::platform::aarch64::cpu::current_el()
    );

    drop(lock);

    unsafe {
        asm!(
            "sub sp, sp, #272",

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
            "str x30, [sp, #16 * 16]",        // offset 256
            "str xzr, [sp, #16 * 16 + 8]",    // pad to 272,

            // We've stored all registers, now lets go to EL0
            "msr SP_EL0, {el0_stack_top}",
            "msr ELR_EL1, {user_func_ptr}",
            "mov  x0, #(1<<9 | 1<<8 | 1<<7)",   // D|A|I masks
            "msr  SPSR_EL1, x0",                // EL0t + DAIF masked

            "isb",
            "eret",

            el0_stack_top = in(reg) el0_sp,
            user_func_ptr = in(reg) user_fp,
        )
    }

    unreachable!();

    // If you get here, we returned from EL0 via SVC and restored EL1 state.
    let _ = writeln!(UartSink, "Back in kernel; x={x}, y={y}, z={z}");
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".user_text")]
pub fn user_func() {
    let mut sp: u64;
    unsafe { asm!("mov {sp}, sp", sp = out(reg) sp) }

    // Runs at EL0
    let x = 33;
    let y = 25;
    let z = String::from("Adios World");

    loop {
        let _ = writeln!(
            UartSink,
            "\"{z}\", from user_func! x = {x}, y = {y} <> [EL0_STACK_TOP = {:x}]",
            sp
        );

        let _ = writeln!(
            UartSink,
            "CurrentEL: {}",
            crate::platform::aarch64::cpu::current_el()
        );

        for _ in 0..10_000_000 {
            unsafe { asm!("nop") }
        }
    }

    // Trigger SVC to return to EL1
    unsafe { asm!("svc #0x1234") };

    // Not reached in this flow
}

// --- EL1/EL0 transition machinery ---

// Save a compact EL1 context frame on SP_EL1 and eret to EL0.
// Callee-saved only (x19..x29, LR), plus saved SP and a resume PC.
#[inline(never)]
pub unsafe fn switch_to_user(user_pc: u64, user_sp: u64) {
    const FRAME_SIZE: usize = 0x70;
    const OFF_SAVED_SP: usize = 0x00;
    const OFF_X19: usize = 0x08;
    const OFF_LR: usize = 0x60;
    const OFF_RESUME_PC: usize = 0x68;

    asm!(
        // --- save EL1 callee-saved frame & set up EL0 ---
        "sub     sp, sp, {frame}",
        "add     x9, sp, {frame}",
        "str     x9, [sp, {off_saved_sp}]",
        "stp     x19, x20, [sp, {off_x19} + 16*0]",
        "stp     x21, x22, [sp, {off_x19} + 16*1]",
        "stp     x23, x24, [sp, {off_x19} + 16*2]",
        "stp     x25, x26, [sp, {off_x19} + 16*3]",
        "stp     x27, x28, [sp, {off_x19} + 16*4]",
        "str     x29,      [sp, {off_x19} + 16*5]",
        "str     x30, [sp, {off_lr}]",
        "adr     x9, 1f",
        "str     x9, [sp, {off_resume_pc}]",
        "msr     sp_el0, {user_sp}",
        "msr     elr_el1, {user_pc}",
        "mov     x0, #0",          // EL0t; set DAIF as you need
        "msr     spsr_el1, x0",
        "isb",
        "eret",                    // -> EL0 (never falls through)

        // We come back here from the SVC handler via `ret x9`
        "1:",
        "ret",                     // return from switch_to_user to caller

        frame = const FRAME_SIZE,
        off_saved_sp = const OFF_SAVED_SP,
        off_x19 = const OFF_X19,
        off_lr  = const OFF_LR,
        off_resume_pc = const OFF_RESUME_PC,
        user_pc = in(reg) user_pc,
        user_sp = in(reg) user_sp,
        // DO NOT use `noreturn` here because we *do* return via label 1.
        options()
    );
}
