#![no_std]
#![feature(asm)]
#![doc(html_root_url = "https://doc.metta.systems/")]


#[cfg(not(any(all(target_arch = "arm", target_pointer_width = "32"),
                  all(target_arch = "x86"), all(target_arch = "x86_64"))))]
use architecture_not_supported_sorry;

#[cfg(target_arch = "x86_64")]
include!("arch/x86_64.rs");

#[cfg(all(target_arch = "arm", target_pointer_width = "64"))]
include!("arch/aarch64.rs");
