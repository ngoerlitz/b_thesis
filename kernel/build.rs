fn main() {
    println!("cargo:rerun-if-changed=asm");
    println!("cargo:rerun-if-changed=bsp");
    println!("cargo:rerun-if-changed=kernel/kernel.ld");
    println!("cargo::rerun-if-env-changed=CARGO_FEATURE_QEMU");

    let qemu = std::env::var_os("CARGO_FEATURE_QEMU").is_some();
    println!("cargo:warning=build.rs: `CARGO_FEATURE_QEMU`: {qemu}");

    let mut build = cc::Build::new();

    build.compiler("aarch64-none-elf-gcc");
    build.archiver("aarch64-none-elf-ar");

    for entry in glob::glob("src/asm/**/*.S").unwrap().filter_map(Result::ok) {
        build.file(entry);
    }

    let pattern = if qemu {
        "src/bsp/rpi4b_qemu/**/*.S"
    } else {
        "src/bsp/rpi4b/**/*.S"
    };

    for entry in glob::glob(pattern).unwrap().filter_map(Result::ok) {
        build.file(entry);
    }

    build.compile("boot");
}
