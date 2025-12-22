.globl _park_secondary

/**
* On the hardware, the secondary cores are parked in a spinlock-table state.
* Therefore, we don't need to manually park these cores here.
*/
_park_secondary:
    ret