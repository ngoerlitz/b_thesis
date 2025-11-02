
.extern __stack_00_el1_top
.extern __stack_01_el1_top
.extern __stack_02_el1_top
.extern __stack_03_el1_top
.extern __mailbox_top

.extern kernel_main
.extern kernel_secondary
.extern _el3

.globl _start
.globl _kmain

.section .init
_start:
    b _el3

_kmain:
    /*
    * Jumps to the kernel entry point in Rust. This requires that we are in the correct
    * EL (EL1) and have valid SPs set. Ideally, we will also have our VBAR_EL1 setup at
    * this point, although this is not a requirement since we can wait until we unmask
    * interrupts.
    */

    mrs     x0, mpidr_el1
    and     x0, x0, #3                  // x0 = core id 0..3

    cbz     x0, _core0
    b _secondary_park


_core0:
    ldr x1, =__stack_00_el1_top
    mov sp, x1

    b kernel_main


_never_wake_up:
    wfe
    b _never_wake_up

_secondary_park:
    mrs x0, MPIDR_EL1
    and x0, x0, #0xFF // x0 = core_aff (core_id)

    // x22 = &cpu_mailbox[]
    adrp x22, __mailbox_top
    add x22, x22, :lo12:__mailbox_top

    // x21 = &cpu_mailbox[cpuid]
    mov x1, #32        // sizeof(CpuMailbox)
    mul x21, x0, x1
    add x21, x22, x21   // __mailbox_top + core_id * sizeof(CpuMailbox)

    wfe

1:
    ldr w2, [x21, #24] // mailbox[2] -> go_flag
    cbz w2, 2f

    ldr x3, [x21, #0]
    mov sp, x3

    ldr x1, [x21, #8]
    ldr x0, [x21, #16]

    dmb ish
    br x1

2:
    wfe
    b 1b