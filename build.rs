// Build file to generate native (asm) code of the boot-loader

extern crate cc;

use std::env;

fn build_libboot(arch: &str, llvmtriple: &str) {
    env::set_var("TARGET_CC", "clang");
    cc::Build::new()
        .target(llvmtriple)
        .file(format!("src/boot/{}.s", arch))
        .flag("-fPIC")
        .flag("-Wno-unused-command-line-argument")
        .compile(arch);
}

fn main() {
    let target = env::var("TARGET").unwrap();
    if target == "i686-vesper-metta" {
        build_libboot("x86", "i686-unknown-linux-gnu");
        println!("cargo:rustc-link-lib=static=x86");
        println!("cargo:rerun-if-changed=src/boot/x86.s");
    } else if target == "x86_64-vesper-metta" {
        build_libboot("x86_64", "x86_64-unknown-linux-gnu");
        println!("cargo:rustc-link-lib=static=x86_64");
        println!("cargo:rerun-if-changed=src/boot/x86_64.s");
    } else if target == "arm-vesper-metta" {
        build_libboot("arm", "arm-linux-gnueabihf");
        println!("cargo:rustc-link-lib=static=arm");
        println!("cargo:rerun-if-changed=src/boot/arm.s");
    } else if target == "aarch64-vesper-metta" {
        build_libboot("aarch64", "aarch64-unknown-linux-gnu");
        println!("cargo:rustc-link-lib=static=aarch64");
        println!("cargo:rerun-if-changed=src/boot/aarch64.s");
        println!("cargo:rerun-if-changed=linker/aarch64.ld");

        // Todo: generate a runner script that would run objcopy from generated kernel.elf to binary
        // This script will be ran by `cargo run` command defined in .cargo/config
        // File::create(out_dir.join("complete.sh")).unwrap()
            // .write_all(b"Script here").unwrap();
    } else {
        panic!("TARGET env variable is not set to one of supported values");
    }
}
