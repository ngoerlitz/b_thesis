.macro ADR_REL register, symbol
	adrp \register, \symbol
	add	 \register, \register, #:lo12:\symbol
.endm

.globl _setup

.section .text._start

.extern _park_secondary
.extern _el3

_start:
    b _el3

_setup:
	// Initialize DRAM.
	ADR_REL	x1, __bss_start
	ADR_REL x2, __bss_end_exclusive

.L_bss_init_loop:
	cmp	x1, x2
	b.eq	.L_prepare_rust
	stp	xzr, xzr, [x1], #16
	b	.L_bss_init_loop

	// Prepare the jump to Rust code.
.L_prepare_rust:
	// Set the stack pointer.
    adrp x2, __kstack_end
    add x2, x2, #:lo12:__kstack_end
    mov sp, x2

    msr daifset, #0xF

	// Jump to Rust code.
	b	__kernel_entry

	// Infinitely wait for events (aka "park the core").
.L_parking_loop:
	wfe
	b	.L_parking_loop

 .size	_start, . - _start
 .type	_start, function
 .global	_start
