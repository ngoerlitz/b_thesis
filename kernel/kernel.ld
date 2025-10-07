/* AArch64 bare-metal (RPi4) */
ENTRY(_start)

/* ---- constants ---- */
STACK_SIZE   = 16K;
NUM_KSTACKS  = 4;
HEAP_SIZE    = 1024M;
PAGE_SIZE    = 4K;

MEMORY {
  SDRAM (rwx) : ORIGIN = 0x0, LENGTH = 2048M
}

/* ---- tops of stacks at end of RAM ---- */
__ram_end          = (ORIGIN(SDRAM) + LENGTH(SDRAM));
__stack_00_el1_top = (__ram_end & ~15);
__stack_01_el1_top = (__stack_00_el1_top - 1 * STACK_SIZE);
__stack_02_el1_top = (__stack_00_el1_top - 2 * STACK_SIZE);
__stack_03_el1_top = (__stack_00_el1_top - 3 * STACK_SIZE);
__stack_el0_top    = (__stack_00_el1_top - 4 * STACK_SIZE);
__stack_bottom = (__stack_el0_top & ~15);

SECTIONS
{
  /* Kernel image load address */
  . = 0x80000;

  /* Keep boot code */
  .init : {
    KEEP(*(.init))
  } > SDRAM

  /* ---- kernel text ---- */
  . = ALIGN(PAGE_SIZE);
  __text_start = .;
  .text : {
    *(.text .text.*)
  } > SDRAM
  __text_end = .;

  /* ---- read-only data ---- */
  . = ALIGN(PAGE_SIZE);
  __rodata_start = .;
  .rodata : {
    *(.rodata .rodata.*)
  } > SDRAM
  __rodata_end = .;

  /* ---- kernel data ---- */
  . = ALIGN(PAGE_SIZE);
  __data_start = .;
  .data : {
    *(.data .data.*)
  } > SDRAM
  __data_end = .;

  /* ---- kernel BSS ---- */
  . = ALIGN(PAGE_SIZE);
  __bss_start = .;
  .bss (NOLOAD) : {
    *(.bss .bss.*)
    *(COMMON)
  } > SDRAM
  __bss_end = .;

    /* ---- user text (run at EL0) ----
       Place any function with:  #[link_section = ".user_text"]
    */
      . = ALIGN(PAGE_SIZE);
      __user_text_start = .;
      .user_text : {
        KEEP(*(.user_text .user_text.*))
      } > SDRAM
      __user_text_end = .;

  /* ---- heap ---- */
  . = ALIGN(8);
  __heap_start = .;
  . = . + HEAP_SIZE;
  __heap_end = .;
}

/* sanity checks */
ASSERT(__heap_end <= (__stack_el0_top - STACK_SIZE),
       "Error: heap overlaps with stacks; reduce heap or stack size");
ASSERT(__heap_end <= __ram_end,
       "Error: heap exceeds SDRAM");
