// Build file to generate native (asm) code of the boot-loader

#[macro_use] extern crate maplit;

use std::process::Command;
use std::env;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    assert!(Command::new("/usr/bin/env")
        .arg("mkdir")
        .arg("-p")
        .arg(&*format!("{}/src/boot", out_dir))
        .status().unwrap().success());

    let arch_to_target = hashmap! {
        "x86" => "i686-unknown-linux-gnu",
        "x86_64" => "x86_64-unknown-linux-gnu",
        "arm" => "arm-linux-gnueabihf",
        "aarch64" => "aarch64-unknown-linux-gnu",
    };
    for (arch, llvmtriple) in &arch_to_target {
        assert!(Command::new("/usr/bin/env")
            .arg("clang")
            .arg("-fPIC")
            .arg(&*format!("src/boot/{}.s", arch))
            .args(&["-c", "-target", llvmtriple, "-o", &*format!("{}/src/boot/{}.o", out_dir, arch)])
            .status().unwrap().success());
        assert!(Command::new("/usr/bin/env")
            .arg("ar")
            .arg("crus")
            .arg(format!("{}/src/boot/lib{}.a", out_dir,arch))
            .arg(&*format!("{}/src/boot/{}.o", out_dir, arch))
            .status().unwrap().success());
    }

    println!("cargo:rustc-link-search=native={}/src/boot", out_dir);
    let target = env::var("TARGET").unwrap();
    if target == "i686-vesper-metta" {
        println!("cargo:rustc-link-lib=static=x86");
    } else if target == "x86_64-vesper-metta" {
        println!("cargo:rustc-link-lib=static=x86_64");
    } else if target == "arm-vesper-metta" {
        println!("cargo:rustc-link-lib=static=arm");
    } else if target == "aarch64-vesper-metta" {
        println!("cargo:rustc-link-lib=static=aarch64");
    } else {
        panic!("TARGET env variable is not set to one of supported values");
    }
}
