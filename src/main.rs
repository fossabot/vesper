#![no_std]
#![no_main]
#![feature(asm)]
#![feature(used)]
#![feature(lang_items)]
#![doc(html_root_url = "https://doc.metta.systems/")]

#[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
use architecture_not_supported_sorry;

#[macro_use]
pub mod arch;
pub use arch::*;

// User-facing kernel parts - syscalls and capability invocations.
// pub mod vesper; -- no mod exported, because available through syscall interface

// Actual interfaces to call these syscalls are in vesper-user (similar to libsel4)
// pub mod vesper; -- exported from vesper-user

#[lang = "eh_personality"]
#[no_mangle]
pub extern fn eh_personality() {}

#[lang = "panic_fmt"]
#[no_mangle]
pub extern fn panic_fmt() -> ! {
    loop {}
}

// Kernel entry point
// arch crate is responsible for calling this
#[no_mangle]
pub extern fn kmain() -> ! {
    loop {}
}
