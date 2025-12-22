.globl _park_secondary

_park_secondary:
    mrs x0, mpidr_el1
    and x0, x0, #0xff

    cbnz x0, .L_secondary
    ret

.L_secondary:
1:
    wfe
    b 1b
