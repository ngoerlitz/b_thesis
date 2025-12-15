.extern el0_sync
.extern el0_irq
.extern el1_irq
.extern el1_serror
.extern err_invalid
.extern unhandled_irq
.extern __stack_el0_top

/*
 * Must by 16 Byte aligned as per AAPCS64
 */
.equ CONTEXT_SIZE, 272

/*
 * This macro stores all of the general purpose registers (x0 - x29) on the stack pointer `sp`
 *
 * 60 Bytes (15 Instructions * 4 Bytes)
 */
.macro STORE_GP_REGS
    // store general purpose registers
    stp x0, x1,   [sp, #16 *  0]
    stp x2, x3,   [sp, #16 *  1]
    stp x4, x5,   [sp, #16 *  2]
    stp x6, x7,   [sp, #16 *  3]
    stp x8, x9,   [sp, #16 *  4]
    stp x10, x11, [sp, #16 *  5]
    stp x12, x13, [sp, #16 *  6]
    stp x14, x15, [sp, #16 *  7]
    stp x16, x17, [sp, #16 *  8]
    stp x18, x19, [sp, #16 *  9]
    stp x20, x21, [sp, #16 * 10]
    stp x22, x23, [sp, #16 * 11]
    stp x24, x25, [sp, #16 * 12]
    stp x26, x27, [sp, #16 * 13]
    stp x28, x29, [sp, #16 * 14]
.endm

/*
 * This macro stores the special registers `ELR_EL1` and `SPSR_EL1` at the offsets matching the `STORE_GP_REGS`
 * macro.
 *
 * 12 Bytes (3 Instructions * 4 Bytes)
 */
.macro STORE_SPECIAL_REGS
    mrs x0, ELR_EL1
    mrs x1, SPSR_EL1
    stp x0, x1,   [sp, #16 * 15]
.endm

/*
 *
 * 20 Bytes (5 Instructions * 4 Bytes)
 * + sizeof(STORE_GP_REGS) = 60
 * + sizeof(STORE_SPECIAL_REGS) = 12
 * = 92 Bytes
 */
.macro ISR_STORE_STACK
    msr DAIFSet, #0b1111 // D/A/I/F mask
    sub sp, sp, #CONTEXT_SIZE

    STORE_GP_REGS
    STORE_SPECIAL_REGS

    // LR (x30) and 8-byte padding to keep 16B alignment
    str x30, [sp, #16 * 16]        // offset 256
    str xzr, [sp, #16 * 16 + 8]    // pad to 272

    // arg0 = &ExceptionFrame
    mov x0, sp
.endm

.macro VECTOR label
    .balign 0x80

    ISR_STORE_STACK

    bl \label
    b .exit_exception
.endm

.macro VECTOR_SYNC label
    .balign 0x80

    ISR_STORE_STACK

    // We are storing the address of the previous SP, which would contain
    // the EL1 execution stack from before the EL1 -> EL0 switch. If the SVC
    // requests an EL0 -> EL1 transition, these values are used to restore the
    // previous EL1 state. In this case, the following `.exit_exception` is
    // NOT called.
    add x1, sp, #CONTEXT_SIZE
    bl \label
    b .exit_exception

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
    VECTOR_SYNC el0_sync           // VBAR_EL1 + 0x400  EL0: Synchronous
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

    eret