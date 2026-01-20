.extern el0_sync
.extern el0_irq
.extern el1_irq
.extern el1_serror
.extern err_invalid
.extern unhandled_irq
.extern __stack_el0_top

.macro VECTOR label
    .balign 0x80
    msr DAIFSet, #0b1111

    // Save all registers (x0 - x30)
    sub sp, sp, #256
    stp x30, xzr, [sp, #240]    // [SP + 240] [SP + 248]
    stp x28, x29, [sp, #224]    // [SP + 224] [SP + 232]
    stp x26, x27, [sp, #208]    // [SP + 208] [SP + 216]
    stp x24, x25, [sp, #192]    // [SP + 192] [SP + 200]
    stp x22, x23, [sp, #176]    // [SP + 176] [SP + 184]
    stp x20, x21, [sp, #160]    // [SP + 160] [SP + 168]
    stp x18, x19, [sp, #144]    // [SP + 144] [SP + 152]
    stp x16, x17, [sp, #128]    // [SP + 128] [SP + 136]
    stp x14, x15, [sp, #112]    // [SP + 112] [SP + 120]
    stp x12, x13, [sp, #96]     // [SP + 96 ] [SP + 104]
    stp x10, x11, [sp, #80]     // [SP + 80 ] [SP + 88 ]
    stp x8, x9,   [sp, #64]     // [SP + 64 ] [SP + 72 ]
    stp x6, x7,   [sp, #48]     // [SP + 48 ] [SP + 56 ]
    stp x4, x5,   [sp, #32]     // [SP + 32 ] [SP + 40 ]
    stp x2, x3,   [sp, #16]     // [SP + 16 ] [SP + 24 ]
    stp x0, x1,   [sp, #0]      // [SP + 0  ] [SP + 8  ]

    sub sp, sp, #16
    mrs x0, ELR_EL1
    mrs x1, SPSR_EL1
    stp x0, x1, [sp, #0]        // [SP + 0  ] [SP + 8  ]

    mov x0, sp

    bl \label
    b .exit_exception
.endm

_el0_irq_entry:
    msr DAIFSet, #0b1111

    // Save all registers (x0 - x30)
    sub sp, sp, #256
    stp x28, x29, [sp, #224]    // [SP + 224] [SP + 232]
    stp x26, x27, [sp, #208]    // [SP + 208] [SP + 216]
    stp x24, x25, [sp, #192]    // [SP + 192] [SP + 200]
    stp x22, x23, [sp, #176]    // [SP + 176] [SP + 184]
    stp x20, x21, [sp, #160]    // [SP + 160] [SP + 168]
    stp x18, x19, [sp, #144]    // [SP + 144] [SP + 152]
    stp x16, x17, [sp, #128]    // [SP + 128] [SP + 136]
    stp x14, x15, [sp, #112]    // [SP + 112] [SP + 120]
    stp x12, x13, [sp, #96]     // [SP + 96 ] [SP + 104]
    stp x10, x11, [sp, #80]     // [SP + 80 ] [SP + 88 ]
    stp x8, x9,   [sp, #64]     // [SP + 64 ] [SP + 72 ]
    stp x6, x7,   [sp, #48]     // [SP + 48 ] [SP + 56 ]
    stp x4, x5,   [sp, #32]     // [SP + 32 ] [SP + 40 ]
    stp x2, x3,   [sp, #16]     // [SP + 16 ] [SP + 24 ]
    stp x0, x1,   [sp, #0]      // [SP + 0  ] [SP + 8  ]
    
    mrs x0, SP_EL0
    stp x30, x0, [sp, #240]    // [SP + 240] [SP + 248]

    sub sp, sp, #16
    mrs x0, ELR_EL1
    mrs x1, SPSR_EL1
    stp x0, x1, [sp, #0]        // [SP + 0  ] [SP + 8  ]

    mov x0, sp
    add x1, sp, #272

    bl el0_irq
1:
    // Unreachable.
    b 1b

// Callee saved registers [x19-x28]
_el0_syscall_entry:
    msr DAIFSet, #0b1111    // Disable interrupts

    // SP at stack entry -> should equal a pointer to the EL1Context
    mov x3, sp

    sub sp, sp, #16
    mrs x15, ELR_EL1
    mrs x16, SPSR_EL1
    stp x15, x16, [sp, #0]    // [SP + 0 ] [SP + 8 ]

    // x19 - x28 => 10 Regs * 8 Bytes = 80 Bytes
    sub sp, sp, #96
    stp x29, x30, [sp, #80] // [SP + 80] [SP + 88]
    stp x27, x28, [sp, #64] // [SP + 64] [SP + 72]
    stp x25, x26, [sp, #48] // [SP + 48] [SP + 56]
    stp x23, x24, [sp, #32] // [SP + 32] [SP + 40]
    stp x21, x22, [sp, #16] // [SP + 16] [SP + 24]
    stp x19, x20, [sp, #0]  // [SP + 0 ] [SP + 8 ]

    mov x0, x0
    mov x1, x1
    mov x2, sp
    mov x3, x3
    bl el0_sync
1:
    // Unreachable.
    b 1b


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
    .balign 0x80                   // VBAR_EL1 + 0x400  EL0: Synchronous
    b _el0_syscall_entry

    .balign 0x80                   // VBAR_EL1 + 0x480  EL0: IRQ
    b _el0_irq_entry

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
    ldp x0, x1, [sp, #0]
    msr ELR_EL1, x0
    msr SPSR_EL1, x1

    add sp, sp, #16

    ldp x0, x1,   [sp, #0]
    ldp x2, x3,   [sp, #16]
    ldp x4, x5,   [sp, #32]
    ldp x6, x7,   [sp, #48]
    ldp x8, x9,   [sp, #64]
    ldp x10, x11, [sp, #80]
    ldp x12, x13, [sp, #96]
    ldp x14, x15, [sp, #112]
    ldp x16, x17, [sp, #128]
    ldp x18, x19, [sp, #144]
    ldp x20, x21, [sp, #160]
    ldp x22, x23, [sp, #176]
    ldp x24, x25, [sp, #192]
    ldp x26, x27, [sp, #208]
    ldp x28, x29, [sp, #224]
    ldp x30, xzr, [sp, #240]

    add sp, sp, #256

    isb
    eret