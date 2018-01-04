#![no_std]
#![no_main]
#![feature(asm)]
#![feature(lang_items)]
#![feature(ptr_internals)] // until we mark with PhantomData instead?
#![doc(html_root_url = "https://docs.metta.systems/")]

#[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
use architecture_not_supported_sorry;

extern crate rlibc;
extern crate bitflags;

use core::panic::PanicInfo;
#[macro_use]
pub mod arch;
pub use arch::*;

// User-facing kernel parts - syscalls and capability invocations.
// pub mod vesper; -- no mod exported, because available through syscall interface

// Actual interfaces to call these syscalls are in vesper-user (similar to libsel4)
// pub mod vesper; -- exported from vesper-user

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // @todo rect() + drawtext("PANIC")?
    loop {}
}

// Kernel entry point
// arch crate is responsible for calling this
#[no_mangle]
pub extern fn kmain() -> ! {
    loop {}
}
