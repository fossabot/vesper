#![no_std]
#![no_main]
#![feature(asm)]
#![feature(const_fn)]
#![feature(lang_items)]
// #![feature(repr_align)]
#![feature(ptr_internals)] // temp
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
extern crate rlibc;

use core::panic::PanicInfo;
#[macro_use]
pub mod arch;
pub use arch::*;
pub mod platform;

use platform::{display::Size2d, vc::VC, uart::MiniUart};

// User-facing kernel parts - syscalls and capability invocations.
// pub mod vesper; -- no mod exported, because available through syscall interface

// Actual interfaces to call these syscalls are in vesper-user (similar to libsel4)
// pub mod vesper; -- exported from vesper-user

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // @todo rect() + drawtext("PANIC")?
    endless_sleep();
}

struct RGB(u32);

impl RGB {
    fn rgb(r: u8, g: u8, b: u8) -> RGB {
        RGB((b as u32) << 16 | (g as u32) << 8 | r as u32)
    }
}

// Kernel entry point
// arch crate is responsible for calling this
pub fn kmain() -> ! {
    let uart = MiniUart::new();
    uart.init();
    uart.puts("Hey there, mini uart talking!");

    if let Ok(mut display) = VC::init_fb(Size2d { x: 800, y: 600 }) {
        display.rect(100, 100, 200, 200, RGB::rgb(255, 255, 255).0);
        display.draw_text(50, 50, "Hello there!", RGB::rgb(128, 192, 255).0);
        // display.draw_text(50, 150, core::fmt("Display width {}", display.width), RGB::rgb(255,0,0).0);

        display.draw_text(150, 50, "RED", RGB::rgb(255, 0, 0).0);
        display.draw_text(160, 60, "GREEN", RGB::rgb(0, 255, 0).0);
        display.draw_text(170, 70, "BLUE", RGB::rgb(0, 0, 255).0);
    }

    uart.puts("Bye, going to sleep now");
    endless_sleep();
}
