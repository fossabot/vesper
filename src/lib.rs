#![no_std]
#![feature(asm)]
#![doc(html_root_url = "https://doc.metta.systems/")]

#[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
use architecture_not_supported_sorry;

#[cfg(target_arch = "x86_64")]
include!("arch/x86_64.rs");

#[cfg(target_arch = "aarch64")]
include!("arch/aarch64.rs");
