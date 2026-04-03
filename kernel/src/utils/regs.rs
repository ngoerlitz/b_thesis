#[macro_export]
macro_rules! save_callee_regs {
    () => {
        r#"
            stp x19, x20, [sp, #-16]!
            stp x21, x22, [sp, #-16]!
            stp x23, x24, [sp, #-16]!
            stp x25, x26, [sp, #-16]!
            stp x27, x28, [sp, #-16]!
            stp x29, x30, [sp, #-16]!
        "#
    };
}
