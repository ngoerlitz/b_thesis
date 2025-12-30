use crate::drivers::pl011::PL011;
use crate::hal::timer::SystemTimerDriver;
use crate::isr::Svc;
use crate::platform::aarch64::{cpu, get_cpu_timer};
use crate::{kprintln, save_gp_regs};
use alloc::format;
use alloc::string::String;
use core::arch::asm;
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
                core_id      = in(reg) cpuid as u64,

                out("x9") _,

                // do not claim nostack (we change sp)
                options(preserves_flags),
            );
        }

        {
            // unsafe {
            //     for _ in 0..5_000_000 {
            //         asm!("nop");
            //     }
            // }

            // If you get here, we returned from EL0 via SVC and restored EL1 state.
            kprintln!("[{cpuid}] EL1; x={x}, y={y}, z={z}");
        }
    }
}

#[inline(always)]
#[unsafe(link_section = ".user_text")]
fn sys_write(buf: *const u8, len: usize) {
    unsafe {
        asm!(
        "svc #{svc}",
        svc = const Svc::PrintMsg as u16,
        in("x0") buf,
        in("x1") len,
        options(nostack, preserves_flags)
        );
    }
}

// Put ALL constant bytes in .user_text (not default .rodata)
#[used]
#[unsafe(link_section = ".user_text")]
static PFX0: [u8; 1] = *b"[";

#[used]
#[unsafe(link_section = ".user_text")]
static PFX1: [u8; 3] = *b"] \"";

#[used]
#[unsafe(link_section = ".user_text")]
static MID0: [u8; 18] = *b"\", from user_func!";

#[used]
#[unsafe(link_section = ".user_text")]
static MID1: [u8; 5] = *b" x = ";

#[used]
#[unsafe(link_section = ".user_text")]
static MID2: [u8; 7] = *b" , y = ";

#[used]
#[unsafe(link_section = ".user_text")]
static MID3: [u8; 21] = *b" <> [EL0_STACK_TOP = ";

#[used]
#[unsafe(link_section = ".user_text")]
static SFX0: [u8; 2] = *b"]\n";

#[repr(C)]
struct Buf<const N: usize> {
    buf: [MaybeUninit<u8>; N],
    len: usize,
}

#[inline(always)]
#[unsafe(link_section = ".user_text")]
fn buf_new<const N: usize>() -> Buf<N> {
    Buf {
        buf: unsafe { MaybeUninit::uninit().assume_init() },
        len: 0,
    }
}

#[inline(always)]
#[unsafe(link_section = ".user_text")]
fn buf_push_byte<const N: usize>(b: &mut Buf<N>, v: u8) {
    if b.len < N {
        b.buf[b.len].write(v);
        b.len += 1;
    }
}

#[inline(always)]
#[unsafe(link_section = ".user_text")]
fn buf_push_bytes<const N: usize>(b: &mut Buf<N>, s: *const u8, n: usize) {
    let mut i = 0;
    while i < n && b.len < N {
        unsafe {
            b.buf[b.len].write(*s.add(i));
        }
        b.len += 1;
        i += 1;
    }
}

#[unsafe(link_section = ".user_text")]
fn buf_push_u64_dec<const N: usize>(b: &mut Buf<N>, mut v: u64) {
    let mut tmp = [0u8; 20];
    let mut n = 0;

    if v == 0 {
        buf_push_byte(b, b'0');
        return;
    }
    while v != 0 {
        tmp[n] = b'0' + (v % 10) as u8;
        n += 1;
        v /= 10;
    }
    while n > 0 {
        n -= 1;
        buf_push_byte(b, tmp[n]);
    }
}

#[unsafe(link_section = ".user_text")]
fn buf_push_i64_dec<const N: usize>(b: &mut Buf<N>, v: i64) {
    if v < 0 {
        buf_push_byte(b, b'-');
        buf_push_u64_dec(b, (0u64).wrapping_sub(v as u64));
    } else {
        buf_push_u64_dec(b, v as u64);
    }
}

#[unsafe(link_section = ".user_text")]
fn buf_push_u64_hex_0x<const N: usize>(b: &mut Buf<N>, v: u64) {
    buf_push_byte(b, b'0');
    buf_push_byte(b, b'x');
    for i in 0..16 {
        let shift = (15 - i) * 4;
        let d = ((v >> shift) & 0xF) as u8;
        let c = if d < 10 { b'0' + d } else { b'a' + (d - 10) };
        buf_push_byte(b, c);
    }
}

#[inline(always)]
#[unsafe(link_section = ".user_text")]
fn buf_ptr_len<const N: usize>(b: &Buf<N>) -> (*const u8, usize) {
    (b.buf.as_ptr() as *const u8, b.len)
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".user_text")]
pub extern "C" fn user_func(cpu_id: u8) {
    let sp: u64;
    unsafe { asm!("mov {}, sp", out(reg) sp) }

    let x: i32 = 33;
    let y: i32 = 25;

    // Also avoid &str for "EL0" unless you place it as bytes too:
    #[used]
    #[unsafe(link_section = ".user_text")]
    static Z: [u8; 3] = *b"EL0";

    let mut b: Buf<192> = buf_new();

    buf_push_bytes(&mut b, PFX0.as_ptr(), PFX0.len());
    buf_push_u64_dec(&mut b, cpu_id as u64);
    buf_push_bytes(&mut b, PFX1.as_ptr(), PFX1.len());
    buf_push_bytes(&mut b, Z.as_ptr(), Z.len());
    buf_push_bytes(&mut b, MID0.as_ptr(), MID0.len());
    buf_push_bytes(&mut b, MID1.as_ptr(), MID1.len());
    buf_push_i64_dec(&mut b, x as i64);
    buf_push_bytes(&mut b, MID2.as_ptr(), MID2.len());
    buf_push_i64_dec(&mut b, y as i64);
    buf_push_bytes(&mut b, MID3.as_ptr(), MID3.len());
    buf_push_u64_hex_0x(&mut b, sp);
    buf_push_bytes(&mut b, SFX0.as_ptr(), SFX0.len());

    let (p, n) = buf_ptr_len(&b);
    sys_write(p, n);

    unsafe { asm!("svc #{svc}", svc = const Svc::ReturnEl1 as u16) };
    loop {}
}
