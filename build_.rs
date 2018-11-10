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

// See https://github.com/rust-lang/rust/blob/653da4fd006c97625247acd7e076d0782cdc149b/src/librustc_target/spec/mod.rs#L1275-L1278
// for possible target triple format.

fn main() {
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let target_vendor = env::var("CARGO_CFG_TARGET_VENDOR").unwrap();
//    let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap();

    match (target_arch.as_str(), target_os.as_str(), target_vendor.as_str()) {
        ("i686", "vesper", "metta") => {
            build_libboot("x86", "i686-unknown-none");
            println!("cargo:rustc-link-lib=static=x86");
            println!("cargo:rerun-if-changed=src/boot/x86.s");
        },
        ("x86-64", "vesper", "metta") => {
            build_libboot("x86_64", "x86_64-unknown-none");
            println!("cargo:rustc-link-lib=static=x86_64");
            println!("cargo:rerun-if-changed=src/boot/x86_64.s");
        },
        ("arm", "vesper", "metta") => {
            build_libboot("arm", "arm-linux-gnueabihf");
            println!("cargo:rustc-link-lib=static=arm");
            println!("cargo:rerun-if-changed=src/boot/arm.s");
        },
        ("aarch64", "vesper", "metta") => {
            build_libboot("aarch64", "aarch64-unknown-none");
            println!("cargo:rustc-link-lib=static=aarch64");
            println!("cargo:rerun-if-changed=src/boot/aarch64.s");
            println!("cargo:rerun-if-changed=linker/aarch64.ld");

            // Todo: generate a runner script that would run objcopy from generated kernel.elf to binary
            // This script will be ran by `cargo run` command defined in .cargo/config
            // File::create(out_dir.join("complete.sh")).unwrap()
            // .write_all(b"Script here").unwrap();
        },
        _ => {
            panic!("Target is not set to one of supported values");
        }
    }
}
