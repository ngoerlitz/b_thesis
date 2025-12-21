__rpi_phys_dram_start_addr = 0;

/* The physical address at which the the kernel binary will be loaded by the Raspberry's firmware */
__rpi_phys_binary_load_addr = 0x80000;

/* ---- constants ---- */
STACK_SIZE   = 16K;
NUM_KSTACKS  = 4;
HEAP_SIZE    = 1024M;
PAGE_SIZE    = 4K;

NUM_USTACKS = 1;
USTACK_SIZE = 16K;

ENTRY(_start)

/* Program headers for ELF loaders (e.g. QEMU -kernel with ELF) */
__stack_00_el1_top = (0x10000);

SECTIONS
{
    . = __rpi_phys_binary_load_addr;

    /***********************************************************************************************
    * Code + RO Data + Global Offset Table
    ***********************************************************************************************/
    .text :
    {
        KEEP(*(.text._start))
        *(.text._start_arguments) /* Constants (or statics in Rust speak) read by _start(). */
        *(.text._start_rust)      /* The Rust entry point */
        *(.text*)                 /* Everything else */
    }

    .rodata : ALIGN(8)
    {
        *(.rodata*)
    }

    /***********************************************************************************************
    * Data + BSS
    ***********************************************************************************************/
    .data :
    {
        *(.data*)
    }

    /* Section is zeroed in pairs of u64. Align start and end to 16 bytes */
    .bss (NOLOAD) : ALIGN(16)
    {
        __bss_start = .;
        *(.bss*);
        . = ALIGN(16);
        __bss_end_exclusive = .;
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

        /* If you want to map EXACTLY one 2MiB L2 block:
           pad the end to the next 2MiB boundary and export __user_end.
           Then map [__user_start, __user_end) as one 2MiB block. */
        . = ALIGN(0x200000);
        __user_end = .;

        /* Safety: ensure the whole user image fits in a single 2MiB block */
        ASSERT((__user_end - __user_start) <= 0x200000, "User image exceeds 2MiB")
}
