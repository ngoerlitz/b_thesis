#!/bin/bash

cargo build && echo "" > qemu.log && \
aarch64-none-elf-objcopy target/aarch64-unknown-none-softfloat/debug/kernel ./kernel8.img && \
qemu-system-aarch64 -machine raspi4b -kernel kernel8.img -nographic -m 2048 -d mmu -D qemu.log
