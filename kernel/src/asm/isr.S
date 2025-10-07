.extern el0_sync
.extern el0_irq
.extern el1_irq
.extern el1_serror
.extern err_invalid
.extern unhandled_irq
.extern __stack_el0_top

.equ CONTEXT_SIZE, 272 // Must by 16 Byte aligned as per AAPCS64

.macro EXCEPTION_VECTOR handler
    sub sp, sp, #CONTEXT_SIZE

    // store general purpose registers
    stp x0, x1, [sp, #16 * 0]
    stp x2, x3, [sp, #16 * 1]
    stp x4, x5, [sp, #16 * 2]
    stp x6, x7, [sp, #16 * 3]
    stp x8, x9, [sp, #16 * 4]
    stp x10, x11, [sp, #16 * 5]
    stp x12, x13, [sp, #16 * 6]
    stp x14, x15, [sp, #16 * 7]
    stp x16, x17, [sp, #16 * 8]
    stp x18, x19, [sp, #16 * 9]
    stp x20, x21, [sp, #16 * 10]
    stp x22, x23, [sp, #16 * 11]
    stp x24, x25, [sp, #16 * 12]
    stp x26, x27, [sp, #16 * 13]
    stp x28, x29, [sp, #16 * 14]


    // ELR/SPSR pair
    mrs x0, ELR_EL1
    mrs x1, SPSR_EL1
    stp x0, x1, [sp, #16 * 15]     // offset 240

    // LR (x30) and 8-byte padding to keep 16B alignment
    str x30, [sp, #16 * 16]        // offset 256
    str xzr, [sp, #16 * 16 + 8]    // pad to 272

    // arg0 = &ExceptionFrame
    mov x0, sp

    bl \handler
    b .exit_exception
.endm

.macro EXCEPTION_VECTOR_NO_RET handler
    msr DAIFSet, #0b1111
    sub sp, sp, #CONTEXT_SIZE

    // store general purpose registers
    stp x0, x1, [sp, #16 * 0]
    stp x2, x3, [sp, #16 * 1]
    stp x4, x5, [sp, #16 * 2]
    stp x6, x7, [sp, #16 * 3]
    stp x8, x9, [sp, #16 * 4]
    stp x10, x11, [sp, #16 * 5]
    stp x12, x13, [sp, #16 * 6]
    stp x14, x15, [sp, #16 * 7]
    stp x16, x17, [sp, #16 * 8]
    stp x18, x19, [sp, #16 * 9]
    stp x20, x21, [sp, #16 * 10]
    stp x22, x23, [sp, #16 * 11]
    stp x24, x25, [sp, #16 * 12]
    stp x26, x27, [sp, #16 * 13]
    stp x28, x29, [sp, #16 * 14]


    // ELR/SPSR pair
    mrs x0, ELR_EL1
    mrs x1, SPSR_EL1
    stp x0, x1, [sp, #16 * 15]     // offset 240

    // LR (x30) and 8-byte padding to keep 16B alignment
    str x30, [sp, #16 * 16]        // offset 256
    str xzr, [sp, #16 * 16 + 8]    // pad to 272

    // arg0 = &ExceptionFrame
    mov x0, sp

    b \handler
.endm

/* ExceptionFrame layout (bytes):
 *   0..247:  x0..x30  (31 * 8)
 * 248..303:  specials (7 * 8) in this order:
 *            sp_el0, elr_el1, spsr_el1, esr_el1, far_el1, tpidr_el0, ttbr0_el1
 * Total: 304 bytes (multiple of 16 to keep SP aligned for AAPCS64).
 */
.equ FRAME_SIZE, 304
.equ ALIGN_MASK_INV, -16      // ~0xF

  // --- Drop to EL0 and execute SVC ---
    .globl drop
drop:
    ldr   x0, =__stack_el0_top
    msr   SP_EL0, x0             // give EL0 a stack

    ldr   x0, =el0_then_svc
    msr   ELR_EL1, x0            // return address = EL0 code

    msr   SPSR_EL1, xzr
    isb

    eret

el0_then_svc:
    svc   #0x42
    b     .

    // ----------------- VECTORS -----------------
    .align 11                    // 2 KiB alignment
    .globl el1_vectors
el1_vectors:
    EXCEPTION_VECTOR err_invalid

.balign 0x80
    EXCEPTION_VECTOR unhandled_irq

.balign 0x80
    EXCEPTION_VECTOR unhandled_irq

.balign 0x80
    EXCEPTION_VECTOR unhandled_irq


.balign 0x80
    EXCEPTION_VECTOR el1_serror

.balign 0x80
    EXCEPTION_VECTOR el1_irq

.balign 0x80
    EXCEPTION_VECTOR unhandled_irq

.balign 0x80
    EXCEPTION_VECTOR unhandled_irq


.balign 0x80
    b el0_sync

.balign 0x80
    EXCEPTION_VECTOR el0_irq

.balign 0x80
    EXCEPTION_VECTOR unhandled_irq

.balign 0x80
    EXCEPTION_VECTOR unhandled_irq


.balign 0x80
    EXCEPTION_VECTOR unhandled_irq

.balign 0x80
    EXCEPTION_VECTOR unhandled_irq

.balign 0x80
    EXCEPTION_VECTOR unhandled_irq

.balign 0x80
    EXCEPTION_VECTOR unhandled_irq

.exit_exception:
    // restore link register
    ldr x30, [sp, #16 * 16]

    // restore exception link register and saved processor state register
    ldp x0, x1, [sp, #16 * 15]
    msr elr_el1, x0
    msr spsr_el1, x1

    // restore general purpose registers
    ldp x28, x29, [sp, #16 * 14]
    ldp x26, x27, [sp, #16 * 13]
    ldp x24, x25, [sp, #16 * 12]
    ldp x22, x23, [sp, #16 * 11]
    ldp x20, x21, [sp, #16 * 10]
    ldp x18, x19, [sp, #16 * 9]
    ldp x16, x17, [sp, #16 * 8]
    ldp x14, x15, [sp, #16 * 7]
    ldp x12, x13, [sp, #16 * 6]
    ldp x10, x11, [sp, #16 * 5]
    ldp x8, x9, [sp, #16 * 4]
    ldp x6, x7, [sp, #16 * 3]
    ldp x4, x5, [sp, #16 * 2]
    ldp x2, x3, [sp, #16 * 1]
    ldp x0, x1, [sp, #16 * 0]

    // restore stack pointer
    add sp, sp, #CONTEXT_SIZE

    eret