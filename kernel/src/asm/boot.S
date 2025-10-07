
.extern __stack_00_el1_top
.extern __stack_01_el1_top
.extern __stack_02_el1_top
.extern __stack_03_el1_top

.extern kernel_main
.extern _el3

.globl _start
.globl _kmain

.section .init
_start:
    b _el3

    // TODO: Read the .dtb pointer (x0) and store this in order to pass it to the
    // TODO: kernel

    // Check processor ID is zero (executing on main core), else hang
    mrs     x1, mpidr_el1
    and     x1, x1, #3
    cbz     x1, 2f
    // We're not on the main core, so hang in an infinite wait loop
1:  wfe
    b       1b
2:  // We're on the main core!
    // Set stack to start below our code

    b _el3

    // Jump to our main() routine in C (make sure it doesn't return)
4:  bl      kernel_main
    // In case it does return, halt the master core too
    b       1b


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
    cmp     x0, #1
    b.eq    _core1
    cmp     x0, #2
    b.eq    _core2
    b       _core3                     // default: core 3

_core0:
    ldr x1, =__stack_00_el1_top
    mov sp, x1

    b kernel_main

_core1:
    ldr x1, =__stack_01_el1_top
    mov sp, x1

    b _secondary_park

_core2:
    ldr x1, =__stack_02_el1_top
    mov sp, x1

    b _secondary_park

_core3:
    ldr x1, =__stack_03_el1_top
    mov sp, x1

    b _secondary_park

.globl WAKEUP_FLAG
.balign 8
WAKEUP_FLAG:
    .quad 0

_secondary_park:
    sevl    // Clear event state
1:
    wfe     // Sleep until event
    adrp x1, WAKEUP_FLAG
    mov  x2, #1
    ldr  x2, [x1, :lo12:WAKEUP_FLAG]
    cbz  x2, 1b
    dmb  ish

    b kernel_main