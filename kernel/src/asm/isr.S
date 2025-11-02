.extern el0_sync
.extern el0_irq
.extern el1_irq
.extern el1_serror
.extern err_invalid
.extern unhandled_irq
.extern __stack_el0_top

.equ CONTEXT_SIZE, 272 // Must by 16 Byte aligned as per AAPCS64

.macro EXCEPTION_VECTOR handler
    msr DAIFSet, #0b1111 // D/A/I/F mask
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

.macro VECTOR label
    .balign 0x80
    EXCEPTION_VECTOR \label
.endm

.macro VECTOR_BR label
    .balign 0x80
    b \label
.endm

.section .text
.align 11
.global el1_vectors

//------------------------------------------------------------------------------
// EL1 Vector Table
// https://developer.arm.com/documentation/102412/0103/Handling-exceptions/Taking-an-exception
//------------------------------------------------------------------------------
// Current EL using SP_EL0
//------------------------------------------------------------------------------
el1_vectors:
    VECTOR unhandled_irq           // VBAR_EL1 + 0x000  EL1t: Synchronous
    VECTOR unhandled_irq           // VBAR_EL1 + 0x080  EL1t: IRQ
    VECTOR unhandled_irq           // VBAR_EL1 + 0x100  EL1t: FIQ
    VECTOR unhandled_irq           // VBAR_EL1 + 0x180  EL1t: SError

//------------------------------------------------------------------------------
// Current EL using SP_ELx (SP_EL1)
//------------------------------------------------------------------------------
    VECTOR unhandled_irq           // VBAR_EL1 + 0x200  EL1h: Synchronous
    VECTOR exc_irq                 // VBAR_EL1 + 0x280  EL1h: IRQ
    VECTOR unhandled_irq           // VBAR_EL1 + 0x300  EL1h: FIQ
    VECTOR exc_serror              // VBAR_EL1 + 0x380  EL1h: SError

//------------------------------------------------------------------------------
// From Lower EL (AArch64)
//------------------------------------------------------------------------------
    VECTOR_BR el0_sync             // VBAR_EL1 + 0x400  EL0: Synchronous
    VECTOR el0_irq                 // VBAR_EL1 + 0x480  EL0: IRQ
    VECTOR unhandled_irq           // VBAR_EL1 + 0x500  EL0: FIQ
    VECTOR unhandled_irq           // VBAR_EL1 + 0x580  EL0: SError

//------------------------------------------------------------------------------
// From Lower EL (AArch32)
//------------------------------------------------------------------------------
    VECTOR unhandled_irq           // VBAR_EL1 + 0x600  EL0 (32): Synchronous
    VECTOR unhandled_irq           // VBAR_EL1 + 0x680  EL0 (32): IRQ
    VECTOR unhandled_irq           // VBAR_EL1 + 0x700  EL0 (32): FIQ
    VECTOR unhandled_irq           // VBAR_EL1 + 0x780  EL0 (32): SError


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

    msr DAIFClr, #0b0111

    eret