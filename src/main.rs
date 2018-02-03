#![no_std]
#![no_main]
#![feature(asm)]
#![feature(used)]
#![feature(lang_items)]
#![feature(attr_literals)]
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

struct Mailbox;

struct VC;

const MAILBOX0READ: u32 = 0x2000b880; // @todo phys2virt
const MAILBOX0STATUS: u32 = 0x2000b898; // @todo phys2virt
const MAILBOX0WRITE: u32 = 0x2000b8a0; // @todo phys2virt

/* Bit 31 set in status register if the write mailbox is full */
const MAILBOX_FULL: u32 = 0x80000000;

/* Bit 30 set in status register if the read mailbox is empty */
const MAILBOX_EMPTY: u32 = 0x40000000;

impl Mailbox {
    pub fn write(channel: u8, physical_base: *const u8) {
        let mailbox_status = MAILBOX0STATUS as *mut u32;
        let mailbox_write = MAILBOX0WRITE as *mut u32;

        while unsafe {*(mailbox_status)} & MAILBOX_FULL != 0 {}
        dmb();
        unsafe {*(mailbox_write) = physical_base as u32 | channel as u32};
    }

    pub fn read(channel: u8) -> Option<u32> {
        let mailbox_status = MAILBOX0STATUS as *mut u32;
        let mailbox_read = MAILBOX0READ as *mut u32;

        let mut count: u32 = 0;
        let mut data: u32 = 0;

        loop {
            while unsafe {*(mailbox_status)} & MAILBOX_EMPTY != 0 {
                count += 1;
                if count > (1 << 25) {
                    return None
                }
            }

            /* Read the data
             * Data memory barriers as we've switched peripheral
             */
            dmb();
            data = unsafe {*(mailbox_read)};
            dmb();

            if (data as u8 & 0xf) == channel {
                return Some(data);
            }
        }
    }
}

struct Size2d {
    x: u32,
    y: u32,
}

impl VC {
    fn get_display_size() -> Option<Size2d> {
        #[repr(align=16)]
        let mut mbox = [0 as u32; 8];

        mbox[0] = 8 * 4;   // Total size
        mbox[1] = 0;       // Request
        mbox[2] = 0x40003; // Display size
        mbox[3] = 8;       // Buffer size
        mbox[4] = 0;       // Request size
        mbox[5] = 0;       // Space for horizontal resolution
        mbox[6] = 0;       // Space for vertical resolution
        mbox[7] = 0;       // End tag

        Mailbox::write(8, &mbox as *const u32 as *const u8); // @todo virt2phys
        if let None = Mailbox::read(8) { // @todo virt2phys
            return None
        }
        if mbox[1] != 0x80000000 {
            return None
        }
        Some(Size2d { x: mbox[5], y: mbox[6] })
    }

    fn set_display_size() {
        #[repr(align=16)]
        let mbox = [0 as u32; 22];
    }

}

// Kernel entry point
// arch crate is responsible for calling this
#[no_mangle]
pub extern fn kmain() -> ! {
    // #[repr(align=16)]
    // let mbox_data = [0 as u32; 128];

    // let gpio = GPIO_BASE as *const u32;
    // let led_on = unsafe { gpio.offset(8) as *mut u32 };
    // let led_off = unsafe { gpio.offset(11) as *mut u32 };

    if let Some(_size) = VC::get_display_size() {}

    // loop {
    //     unsafe { *(led_on) = 1 << 15; }
    //     sleep(500000);
    //     unsafe { *(led_off) = 1 << 15; }
    //     sleep(500000);
    // }
    loop {}
}
