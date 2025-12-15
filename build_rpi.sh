#!/bin/bash

cargo build --target aarch64-unknown-none-softfloat && \
mkdir -p ./target/out && : > ./target/out/qemu.log && \
aarch64-none-elf-objcopy \
  -O binary \
  target/aarch64-unknown-none-softfloat/debug/kernel-playground \
  ./target/out/kernel8.img

cp ./target/out/kernel8.img /media/ngoerlitz/bootfs/kernel8-rpi.img
