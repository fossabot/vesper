#!/bin/sh

aarch64-unknown-linux-musl-objcopy -O binary $1 $1.bin

/usr/local/Cellar/qemu/HEAD-160e5c2-custom/bin/qemu-system-aarch64 -M raspi3 -d in_asm -serial null -serial stdio -kernel $1.bin
