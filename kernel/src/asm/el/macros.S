/*
* CHECK_EL <n> <label>: if CurrentEL!=n, branch to <label>
* Note: CurrentEL's values (i.e. the current EL) is in bits [3:2]
*/
.macro CHECK_NOT_EL n target
    mrs  x1, CurrentEL         // Read the current exception level
    cmp  x1, #\n               // Compare EL to value
    b.ne \target               // If EL is not value, we branch to target
.endm
