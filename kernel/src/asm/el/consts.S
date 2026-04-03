.equ CurrentEL_EL0, 0x0     // 0b0000 - EL0
.equ CurrentEL_EL1, 0x4     // 0b0100 - EL1
.equ CurrentEL_EL2, 0x8     // 0b1000 - EL2
.equ CurrentEL_EL3, 0xC     // 0b1100 - EL3

/*
* Bit definitions for the SPSR_EL3 (Saved Program Status Register; EL3)
* Source: https://developer.arm.com/documentation/ddi0601/2025-06/AArch64-Registers/SPSR-EL3--Saved-Program-Status-Register--EL3-
*/
.equ SPSR_F, (1 << 6)
.equ SPSR_I, (1 << 7)
.equ SPSR_A, (1 << 8)
.equ SPSR_D, (1 << 9)

.equ SPSR_EL2h, 0x9         // 0b1101 - EL2 with SP_EL2 (EL2h)
.equ SPSR_EL1h, 0x5         // 0b0101 - EL1 with SP_EL1 (EL1h)
