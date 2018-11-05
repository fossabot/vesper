// Build file to generate native (asm) code of the boot-loader

#[macro_use] extern crate maplit;

use std::process::Command;
use std::path::PathBuf;
// use std::io::Write;
use std::fs;
use std::env;

fn build_libboot(out_dir: &PathBuf, arch: &str, llvmtriple: &str) {
    // Todo: use gcc/clang crate for this?
    assert!(Command::new("/usr/bin/env")
        .arg("clang")
        .arg("-fPIC")
        .arg(&*format!("src/boot/{}.s", arch))
        .args(&["-c", "-target", llvmtriple, "-o", &*format!("{}/src/boot/{}.o", out_dir.display(), arch)])
        .status().unwrap().success());
    assert!(Command::new("/usr/bin/env")
        .arg("ar")
        .arg("crus")
        .arg(format!("{}/src/boot/lib{}.a", out_dir.display(), arch))
        .arg(&*format!("{}/src/boot/{}.o", out_dir.display(), arch))
        .status().unwrap().success());
}

fn main() {
    let out_dir = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    fs::create_dir_all(format!("{}/src/boot", out_dir.display())).expect("Couldn't create boot directory");

    println!("cargo:rustc-link-search=native={}/src/boot", out_dir.display());
    let target = env::var("TARGET").unwrap();
    if target == "i686-vesper-metta" {
        build_libboot(out_dir, "x86", "i686-unknown-linux-gnu");
        println!("cargo:rustc-link-lib=static=x86");
        println!("cargo:rerun-if-changed=src/boot/x86.s");
    } else if target == "x86_64-vesper-metta" {
        build_libboot(out_dir, "x86_64", "x86_64-unknown-linux-gnu");
        println!("cargo:rustc-link-lib=static=x86_64");
        println!("cargo:rerun-if-changed=src/boot/x86_64.s");
    } else if target == "arm-vesper-metta" {
        build_libboot(out_dir, "arm", "arm-linux-gnueabihf");
        println!("cargo:rustc-link-lib=static=arm");
        println!("cargo:rerun-if-changed=src/boot/arm.s");
    } else if target == "aarch64-vesper-metta" {
        build_libboot(out_dir, "aarch64", "aarch64-unknown-linux-gnu");
        // println!("cargo:rustc-link-search={}", out_dir.display());
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
