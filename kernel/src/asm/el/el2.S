.include "src/asm/el/macros.S"
.include "src/asm/el/consts.S"

.globl _el2
.extern __stack_el0_top

/*
* Bit definitions for the HCR_EL2 (Hypervisor Configuration Register)
* Source: https://developer.arm.com/documentation/ddi0601/2025-06/AArch64-Registers/HCR-EL2--Hypervisor-Configuration-Register
*/
.equ HCR_RW, (1 << 31)

/*
* Bit definitions for the CNTHCTL_EL2 (Counter-timer Hypervisor Control Register)
* Source: https://developer.arm.com/documentation/ddi0601/2025-06/AArch64-Registers/CNTHCTL-EL2--Counter-timer-Hypervisor-Control-Register
*/
.equ EL1PCTEN, (1 << 0)
.equ EL1PCEN,  (1 << 1)


_el2:
    CHECK_NOT_EL CurrentEL_EL2, _el1

    // Set execution state for EL1 to AArch64
    mov x0, #HCR_RW
    msr HCR_EL2, x0
    isb

    // Allow EL1 to read CNTPCT/Phys-Timer
    mrs x0, CNTHCTL_EL2
    orr x0, x0, #(EL1PCEN|EL1PCTEN)
    msr CNTHCTL_EL2, x0
    msr CNTVOFF_EL2, xzr
    isb

    // Don't trap FP/SIMD/PMU/debug to EL2
    msr CPTR_EL2, xzr
    msr MDCR_EL2, xzr
    msr HSTR_EL2, xzr
    isb

    // VBAR_EL1
    // TODO: This needs to be done PER CORE? (Justus?)
    ldr x0, =el1_vectors
    msr VBAR_EL1, x0
    isb

    msr SPSel, #1

    // Return to EL1h with DAIF masked
    mov x0, #(SPSR_D|SPSR_A|SPSR_I|SPSR_F|SPSR_EL1h)
    msr SPSR_EL2, x0

    // Set return addr to EL1
    adr x0, _el1
    msr ELR_EL2, x0
    isb

    eret
