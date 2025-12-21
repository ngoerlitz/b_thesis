#!/bin/bash

cargo build --target aarch64-unknown-none-softfloat && \
mkdir -p ./target/out && \
aarch64-none-elf-objcopy \
  -O binary \
  target/aarch64-unknown-none-softfloat/debug/kernel-playground \
  ./target/out/kernel8.img

sudo cp ./target/out/kernel8.img /srv/tftp/my.img
