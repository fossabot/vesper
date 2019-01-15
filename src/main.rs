#![no_std]
#![no_main]
#![feature(asm)]
#![feature(const_fn)]
#![feature(lang_items)]
#![feature(ptr_internals)] // until we mark with PhantomData instead?
#![feature(core_intrinsics)]
#![doc(html_root_url = "https://docs.metta.systems/")]
#![allow(dead_code)]
#![allow(unused_assignments)]

#[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
use architecture_not_supported_sorry;

// use core::intrinsics::abort;

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate register;
extern crate cortex_a;
extern crate rlibc;

use core::panic::PanicInfo;
#[macro_use]
pub mod arch;
pub use arch::*;
pub mod platform;

use platform::uart::MiniUart;

// User-facing kernel parts - syscalls and capability invocations.
// pub mod vesper; -- no mod exported, because available through syscall interface

// Actual interfaces to call these syscalls are in vesper-user (similar to libsel4)
// pub mod vesper; -- exported from vesper-user

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // @todo rect() + drawtext("PANIC")?
    endless_sleep()
}

// Kernel entry point
// arch crate is responsible for calling this
pub fn kmain() -> ! {
    let uart = MiniUart::new();
    uart.init();
    uart.puts("Hey there, mini uart talking!");

    uart.puts("Bye, going to sleep now");
    endless_sleep()
}
