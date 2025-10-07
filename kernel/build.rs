fn main() {
    println!("cargo:rerun-if-changed=asm/boot.S");
    println!("cargo:rerun-if-changed=kernel/kernel.ld");

    cc::Build::new()
        .files([
            "src/asm/boot.S",
            "src/asm/isr.S",
            "src/asm/el/el.S",
            "src/asm/el/el2.S",
            "src/asm/el/el3.S"
        ])
        .compiler("aarch64-none-elf-gcc")
        .archiver("aarch64-none-elf-ar")
        .compile("boot");
}