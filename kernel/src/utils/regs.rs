/// Saves the general purpose registers (x0 - x29) on the stack identified
/// by `sp`.
/// This will "occupy" the stack slots 0 - 14, i.e. [0; 240[ Bytes offset.
#[macro_export]
macro_rules! save_gp_regs {
    () => {
        r#"
        stp x0, x1,   [sp, #16 *  0]
        stp x2, x3,   [sp, #16 *  1]
        stp x4, x5,   [sp, #16 *  2]
        stp x6, x7,   [sp, #16 *  3]
        stp x8, x9,   [sp, #16 *  4]
        stp x10, x11, [sp, #16 *  5]
        stp x12, x13, [sp, #16 *  6]
        stp x14, x15, [sp, #16 *  7]
        stp x16, x17, [sp, #16 *  8]
        stp x18, x19, [sp, #16 *  9]
        stp x20, x21, [sp, #16 * 10]
        stp x22, x23, [sp, #16 * 11]
        stp x24, x25, [sp, #16 * 12]
        stp x26, x27, [sp, #16 * 13]
        stp x28, x29, [sp, #16 * 14]
        "#
    };
}
