.globl _secondary_release
.extern _secbt
.extern __kstack_end
.extern KSTACK_SIZE

_secondary_release:
    mrs x0, mpidr_el1
    and x0, x0, #0xFF

    // Load stack for core (this is __KSTACK_BASE - (__KSTACK_SIZE * CoreID))
    adrp x1, __kstack_end
    add x1, x1, :lo12:__kstack_end

    ldr x2, =KSTACK_SIZE

    msub x3, x0, x2, x1
    mov sp, x3

    // Set interrupts (daifset, #0xF)
    msr DAIFSet, #0xF

    // Jump to secondary kernel code
    // Moves for clarity
    mov x0, x0          // x0 = CoreID
    b _secbt