.globl _park_secondary
.globl _secondary_release


/**
* On the hardware, the secondary cores are parked in a spinlock-table state.
* Therefore, we don't need to manually park these cores here.
*/
_park_secondary:
    ret

_secondary_release:
    // Load stack for core (this is __KSTACK_BASE - (__KSTACK_SIZE * CoreID))
    adrp x1, __kstack_end
    add x1, x1, :lo12:__kstack_end

    adrp x2, KSTACK_SIZE
    add x2, x2, :lo12:KSTACK_SIZE

    msub x3, x0, x2, x1
    mov sp, x3

    // Set interrupts (daifset, #0xF)
    msr DAIFSet, #0xF

    // Jump to secondary kernel code
    // Moves for clarity
    mov x0, x0          // x0 = CoreID
    b _secbt