.globl _secondary_release
.globl _secbt
.extern __stack_el1_top
.extern __stack_el1_size

/*
_park_secondary:
    mrs x0, mpidr_el1
    and x0, x0, #0x3            // CoreID

    cbnz x0, .L_secondary       // CoreID == 0
    ret

.L_secondary:
    // Calculate Core Release ADDR
    // Base = 0xD8 + (8 * CoreID)

    mov x1, #0xD8       // Base Addr
    mov x2, #0x8        // Stride
    madd x3, x0, x2, x1 // x3 = (CoreID * Stride) + Base Addr

1:
    ldr x4, [x3]        // Load rel addr
    cbz x4, 2f          // If zero continue waiting
    b .L_release

2:
    b 1b
    */

_secondary_release:
    // Load stack for core (this is __KSTACK_BASE - (__KSTACK_SIZE * CoreID))
    adrp x1, __stack_el1_top
    add x1, x1, :lo12:__stack_el1_top

    adrp x2, __stack_el1_size
    add x2, x2, :lo12:__stack_el1_size

    msub x3, x0, x2, x1
    mov sp, x3

    // Set interrupts (daifset, #0xF)
    msr DAIFSet, #0xF

    // Jump to secondary kernel code
    // Moves for clarity
    mov x0, x0          // x0 = CoreID
    b _secbt