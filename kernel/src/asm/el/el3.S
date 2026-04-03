.include "src/asm/el/macros.S"
.include "src/asm/el/consts.S"

.globl _el3

/*
* Bit definitions for the SCR_EL3 (Secure Configuration Register).
* Source: https://developer.arm.com/documentation/ddi0595/2021-06/AArch64-Registers/SCR-EL3--Secure-Configuration-Register?lang=en
*/
.equ SCR_NS,  (1 << 0)
.equ SCR_IRQ, (1 << 1)
.equ SCR_FIQ, (1 << 2)
.equ SCR_EA,  (1 << 3)
.equ SCR_SMD, (1 << 7)
.equ SCR_HCE, (1 << 8)
.equ SCR_SIF, (1 << 9)
.equ SCR_RW,  (1 << 10)
.equ SCR_ST,  (1 << 11)

_el3:
    CHECK_NOT_EL CurrentEL_EL3, _el2

    msr DAIFSet, #0xF           // Mask async exceptions

    mrs x0, SCR_EL3
    bic x0, x0, #SCR_IRQ        // do not route IRQ to EL3
    bic x0, x0, #SCR_FIQ        // do not route FIQ to EL3
    bic x0, x0, #SCR_EA         // do not route SError to EL3
    bic x0, x0, #SCR_ST         // don't trap NS phys timer to EL3
    bic x0, x0, #SCR_SIF        // don’t constrain NS fetches to Secure mem
    orr x0, x0, #SCR_NS         // drop to Non-secure
    orr x0, x0, #SCR_RW         // lower EL in AArch64
    orr x0, x0, #SCR_HCE        // enable HVC at NS EL1 (so EL2 can get it)
    orr x0, x0, #SCR_SMD        // SMC from NS won’t reach EL3
    msr SCR_EL3, x0
    isb

    mov x0, #(SPSR_F|SPSR_A|SPSR_I|SPSR_F|SPSR_EL2h)
    msr SPSR_EL3, x0
    isb

    adr x0, _el2
    msr ELR_EL3, x0
    isb

    eret
