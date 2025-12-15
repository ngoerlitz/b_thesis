#!/bin/bash



cargo build && echo "" > qemu.log && mkdir -p ./target/out && \
aarch64-none-elf-objcopy target/aarch64-unknown-none-softfloat/debug/kernel-playground ./target/out/kernel8.img && \
qemu-system-aarch64 -machine raspi4b -kernel ./target/out/kernel8.img -nographic -m 2048 -d mmu -D ./target/out/qemu.log -s -S
