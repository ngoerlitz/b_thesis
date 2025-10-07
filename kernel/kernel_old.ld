ENTRY(_start)

/* CONSTANT VALUES */
STACK_SIZE      = 16K;
NUM_KSTACKS     = 4;

HEAP_SIZE       = 1024M;

MEMORY
{
    SDRAM  (rwx) : ORIGIN = 0x0, LENGTH = 2048M
}

__ram_end = (ORIGIN(SDRAM) + LENGTH(SDRAM));
__stack_00_el1_top = (__ram_end & ~15);
__stack_01_el1_top = (__stack_00_el1_top - 1 * STACK_SIZE);
__stack_02_el1_top = (__stack_00_el1_top - 2 * STACK_SIZE);
__stack_03_el1_top = (__stack_00_el1_top - 3 * STACK_SIZE);
__stack_el0_top    = (__stack_00_el1_top - 4 * STACK_SIZE);

SECTIONS
{
    . = 0x80000;
    .init : {
        KEEP(*(.init))
    } > SDRAM

    .text : {
        *(.text*)
    } > SDRAM

    .rodata : { *(.rodata*) } > SDRAM
    .data   : { *(.data*)   } > SDRAM

    .bss (NOLOAD) : {
        __bss_start = .;
        *(.bss*)
        *(COMMON)
        __bss_end = .;
    } > SDRAM

    /* Heap grows upward from here */
    . = ALIGN(8);
    __heap_start = .;
    . = . + HEAP_SIZE;
    __heap_end = .;
}

ASSERT(__heap_end <= (__stack_el0_top - STACK_SIZE), "Error: heap overlaps with stacks; reduce heap or stack size");
ASSERT(__heap_end <= __ram_end, "Error: heap overlaps with stacks; reduce heap or stack size");
