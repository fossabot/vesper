#!/bin/sh

aarch64-unknown-linux-musl-objcopy -O binary $1 $1.bin

qemu-system-aarch64 -M raspi3 -d in_asm -serial stdio -kernel $1.bin
