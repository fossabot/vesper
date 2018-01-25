#![no_std]
#![feature(asm)]
#![doc(html_root_url = "https://doc.metta.systems/")]

#[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
use architecture_not_supported_sorry;

#[cfg(target_arch = "x86_64")]
include!("arch/x86_64.rs");

#[cfg(target_arch = "aarch64")]
include!("arch/aarch64.rs");

// User-facing kernel parts - syscalls and capability invocations.
// pub mod vesper; -- no mod exported, because available through syscall interface

// Actual interfaces to call these syscalls are in vesper-user (similar to libsel4)
// pub mod vesper; -- exported from vesper-user
