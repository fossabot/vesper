# Prerequisites

LLD in Rust:
* https://github.com/rust-lang/rust/issues/39915
* https://github.com/rust-lang/rust/pull/40018 `-Z linker-flavor`

The latter also describes steps to build baremetal apps with xargo and lld, quote:

The place where LLD shines is at linking Rust programs that don't depend on
system libraries. For example, here's how you would link a bare metal ARM
Cortex-M program:

```
$ xargo rustc --target thumbv7m-none-eabi -- -Z linker-flavor=ld -C linker=ld.lld -Z print-link-args
"ld.lld" \
  "-L" \
  "$XARGO_HOME/lib/rustlib/thumbv7m-none-eabi/lib" \
  "$PWD/target/thumbv7m-none-eabi/debug/deps/app-de1f86df314ad68c.0.o" \
  "-o" \
  "$PWD/target/thumbv7m-none-eabi/debug/deps/app-de1f86df314ad68c" \
  "--gc-sections" \
  "-L" \
  "$PWD/target/thumbv7m-none-eabi/debug/deps" \
  "-L" \
  "$PWD/target/debug/deps" \
  "-L" \
  "$XARGO_HOME/lib/rustlib/thumbv7m-none-eabi/lib" \
  "-Bstatic" \
  "-Bdynamic" \
  "$XARGO_HOME/lib/rustlib/thumbv7m-none-eabi/lib/libcore-11670d2bd4951fa7.rlib"

$ file target/thumbv7m-none-eabi/debug/app
app: ELF 32-bit LSB executable, ARM, EABI5 version 1 (SYSV), statically linked, not stripped, with debug_info
```

This doesn't require installing the arm-none-eabi-gcc toolchain.

Linker is located in `/usr/local/opt/llvm/bin/lld` in brewed llvm.

A cross-compiler for linux is called `gcc-aarch64-linux-gnu` (Debian).
(That's armv8)

Device tree data is in `arch/arm64/boot/dts/broadcom/bcm2710-rpi-3-b.dtb`

# Triple structure
```
pub struct Triple {
    arch: Arch, // x86, arm, etc
    sub_arch: Option<SubArch>, // mostly for arm
    vendor: Vendor, // apple, pc, etc
    os: Os, // linux, darwin, windows
    environment: Environment, // gnueabihf, musl
}
```

`aarch64-unknown-metta` should be ok for start (but later - target seem to require changing rustc source).

# Building the cross-compiler

Follow https://akappel.github.io/2017/11/07/rpi-crosstool.html

* `rustup target add aarch64-unknown-linux-gnu`
