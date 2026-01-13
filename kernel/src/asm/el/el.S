/*
* The AArch64 arch has 4 exception levels, of which the upper two (EL3, EL2) are optionally
* implemented. Typically, EL3 is used for firmware, whereas EL2 is used for virtualization.
* Since we don't require either, we are going to drop from the current ELx down to EL1 and
* start the kernel from there. This needs to be executed on EVERY core, as the SPSR is not
* shared between these.
*
* Note on EL3:
* Real hardware (especially the Raspi4B) mostly won't implement this EL. Unfortunately for us
* QEMU does simulate it and boots in EL3 (with the -kernel kernel8.img flag). Therefore, we do
* need to provide the handler for EL3.
*/

.globl _el1

.extern _setup
.extern _secondary_release

_el1:
    mrs x0, mpidr_el1
    and x0, x0, #0xFF

    cbnz x0, 1f

    b _setup

1:
    adrp x1, _secondary_release
    add x1, x1, :lo12:_secondary_release
    br x1