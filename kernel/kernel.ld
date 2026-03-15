__rpi_phys_dram_start_addr = 0;

/* The physical address at which the the kernel binary will be loaded by the Raspberry's firmware */
__rpi_phys_binary_load_addr = 0x80000;

/* HEAP = [1GB - 2GB[ */
__heap_start = __user_end;
__heap_end = 0x40000000 - 0x08000000; /* 128MiB rpi reserved */


/* ---- constants ---- */
NUM_CORES = 4;
NUM_USTACKS = 16;

USTACK_SIZE = 0x300000; /* 3MiB */
KSTACK_SIZE = 0x800000; /* 8MiB */

__ustack_size = USTACK_SIZE;


ENTRY(_start)

SECTIONS
{
    . = __rpi_phys_binary_load_addr;

    /***********************************************************************************************
    * Code + RO Data + Global Offset Table
    ***********************************************************************************************/
    .text._start :
    {
        KEEP(*(.text._start))
    }

    . = ALIGN(0x200000);
    .text : {
        __kernel_text_start = .;
        *(.text*)                 /* Everything else */
        . = ALIGN(0x200000);
        __kernel_text_end = .;
    }

    .rodata : ALIGN(8)
    {
        __kernel_rodata_start = .;
        *(.rodata*)
        . = ALIGN(0x200000);
        __kernel_rodata_end = .;
    }

    /***********************************************************************************************
    * Data + BSS
    ***********************************************************************************************/
    .data :
    {
        __kernel_data_start = .;
        *(.data*)
        . = ALIGN(0x200000);
        __kernel_data_end = .;
    }

    /* Section is zeroed in pairs of u64. Align start and end to 16 bytes */
    .bss (NOLOAD) : ALIGN(16)
    {
        __bss_start = .;
        *(.bss*);
        . = ALIGN(16);
        __bss_end_exclusive = .;
    }

    .kernel_stack (NOLOAD) : ALIGN(0x200000)
    {
        . = . + (KSTACK_SIZE * NUM_CORES);
        __kstack_end = .;
    }

    /***********************************************************************************************
    * Misc
    ***********************************************************************************************/
    .got :
    {
        *(.got*)
    }
    ASSERT(SIZEOF(.got) == 0, "Relocation support not expected")

    /DISCARD/ : { *(.comment*) }

    /***********************************************************************************************
    * User image (EL0) - keep contiguous and 2MiB-aligned for single L2 block mapping
    ***********************************************************************************************/

    /* Start of the user-mapped region: MUST be 2MiB aligned */
    . = ALIGN(0x200000);
    __user_start = .;

    /* User code */
    .user_text : ALIGN(16)
    {
        __user_text_start = .;
        KEEP(*(.user_text .user_text.*))
        __user_text_end = .;
    }

    .user_stack (NOLOAD) : ALIGN(16)
    {
        __ustack_start = .;
        . = . + (USTACK_SIZE * NUM_USTACKS);
        __ustack_end = .;
    }

    __stack_el0_top = __ustack_end;
    . = ALIGN(0x200000);
    __user_end = .;

    /* Safety: ensure the whole user image fits in a single 2MiB block */
    /*ASSERT((__user_end - __user_start) <= 0x16000000, "User image exceeds 2MiB") */
}
