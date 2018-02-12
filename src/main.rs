#![no_std]
#![no_main]
#![feature(asm)]
#![feature(used)]
#![feature(const_fn)]
#![feature(lang_items)]
#![feature(attr_literals)]
#![doc(html_root_url = "https://docs.metta.systems/")]

#[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
use architecture_not_supported_sorry;

extern crate rlibc;
#[macro_use]
extern crate bitflags;

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

// See BCM2835-ARM-Peripherals.pdf
// See https://www.raspberrypi.org/forums/viewtopic.php?t=186090 for more details.

// Physical memory is 0x00000000 to 0x40000000
const fn phys2virt(address: u32) -> u32 {
    return address;// + 0x80000000;
}

// RAM bus address is 0xC0000000 to 0xFFFFFFFF
// Peripheral bus memory is 0x7E000000 to 0x7EFFFFFF
const fn phys2bus(address: u32) -> u32 {
    return address;// + (0x40000000); // L2 cache enabled
    // return address.wrapping_add(0xC0000000); // L2 cache disabled
}

const fn bus2phys(address: u32) -> u32 {
    return address;// - (0x40000000); // L2 cache enabled
    // return address.wrapping_sub(0xC0000000); // L2 cache disabled
}

// Identity mapped first 1Gb by uboot
const MAILBOX0READ: u32 = phys2virt(0x3f00b880) as u32; // @todo phys2virt
const MAILBOX0STATUS: u32 = phys2virt(0x3f00b898) as u32; // @todo phys2virt
const MAILBOX0WRITE: u32 = phys2virt(0x3f00b8a0) as u32; // @todo phys2virt

// const MAILBOX_PHYSADDR: u32 = 0x2000b880; // verified: u-boot arch/arm/mach-bcm283x/include/mach/mbox.h

// struct MailboxRegs {
//     read: u32,
//     rsvd0: [u32; 5],
//     status: u32,
//     config: u32,
//     write: u32,
// }

/* Lower 4-bits are channel ID */
const CHANNEL_MASK: u8 = 0xf;

#[repr(u8)]
enum Channel {
    Tags = 8,
}

const MAILBOX_REQ_CODE: u32 = 0;
const MAILBOX_RESP_CODE_SUCCESS: u32 = 0x80000000;

/* When responding, the VC sets this bit in val_len to indicate a response */
const MAILBOX_TAG_VAL_LEN_RESPONSE: u32 = 0x80000000;

#[allow(dead_code)]
enum Tag {
    GetBoardRev = 0x00010002,
    GetMacAddress = 0x00010003,
    GetBoardSerial = 0x00010004,
    GetArmMemory = 0x00010005,
    GetPowerState = 0x00020001,
    SetPowerState = 0x00028001,
    GetClockRate = 0x00030002,
    AllocateBuffer = 0x00040001,
    ReleaseBuffer = 0x00048001,
    BlankScreen = 0x00040002,
    /* Physical means output signal */
    GetPhysicalWH = 0x00040003,
    TestPhysicalWH = 0x00044003,
    SetPhysicalWH = 0x00048003,
    /* Virtual means display buffer */
    GetVirtualWH = 0x00040004,
    TestVirtualWH = 0x00044004,
    SetVirtualWH = 0x00048004,
    GetDepth = 0x00040005,
    TestDepth = 0x00044005,
    SetDepth = 0x00048005,
    GetPixelOrder = 0x00040006,
    TestPixelOrder = 0x00044006,
    SetPixelOrder = 0x00048006,
    GetAlphaMode = 0x00040007,
    TestAlphaMode = 0x00044007,
    SetAlphaMode = 0x00048007,
    GetPitch = 0x00040008,
    /* Offset of display window within buffer */
    GetVirtualOffset = 0x00040009,
    TestVirtualOffset = 0x00044009,
    SetVirtualOffset = 0x00048009,
    GetOverscan = 0x0004000a,
    TestOverscan = 0x0004400a,
    SetOverscan = 0x0004800a,
    GetPalette = 0x0004000b,
    TestPalette = 0x0004400b,
    SetPalette = 0x0004800b,
    End = 0,
}


/* Bit 31 set in status register if the write mailbox is full */
const MAILBOX_STATUS_WR_FULL: u32 = 0x80000000;

/* Bit 30 set in status register if the read mailbox is empty */
const MAILBOX_STATUS_RD_EMPTY: u32 = 0x40000000;

impl Mailbox {
    pub fn write(channel: u8, physical_base: *const u8) {
        let mailbox_status = MAILBOX0STATUS as *mut u32;
        let mailbox_write = MAILBOX0WRITE as *mut u32;
        // let mailbox = MAILBOX_PHYSADDR as *mut MailboxRegs;
        let mut count: u32 = 0;

        while unsafe { *(mailbox_status) & MAILBOX_STATUS_WR_FULL != 0 } {
            flushcache(MAILBOX0STATUS as usize);
            count += 1;
            if count > (1 << 25) {
                return
            }
        }
        dmb();
        unsafe { *(mailbox_write) = phys2bus(physical_base as u32) | channel as u32 };
    }

    pub fn read(channel: u8) -> Option<u32> {
        let mailbox_status = MAILBOX0STATUS as *mut u32;
        let mailbox_read = MAILBOX0READ as *mut u32;
        // let mailbox = MAILBOX_PHYSADDR as *mut MailboxRegs;
        let mut count: u32 = 0;

        loop {
            while unsafe {*(mailbox_status)} & MAILBOX_STATUS_RD_EMPTY != 0 {
                flushcache(MAILBOX0STATUS as usize);
                count += 1;
                if count > (1 << 25) {
                    return None
                }
            }

            /* Read the data
             * Data memory barriers as we've switched peripheral
             */
            dmb();
            let data = unsafe {*(mailbox_read)};
            dmb();

            if (data as u8 & CHANNEL_MASK) == channel {
                return Some(data);
            }
        }
    }

    pub fn call(channel: u8, physical_base: *const u8) -> Option<u32> {
        Self::write(channel, physical_base);
        Self::read(channel)
    }
}

struct Size2d {
    x: u32,
    y: u32,
}

/* Character cells are 6x10 */
const CHARSIZE_X: u32 = 6;
const CHARSIZE_Y: u32 = 10;

struct Display {
    base: u32,
    size: u32,
    pitch: u32,
    max_x: u32,
    max_y: u32,
}

// bufsize
// code
// ....
// end tag

// tag code
// val bufsize
// val size
// ...data buf

impl VC {
    fn get_display_size() -> Option<Size2d> {
        #[repr(align=16)]
        let mut mbox = [0 as u32; 8];

        mbox[0] = 8 * 4;   // Total size
        mbox[1] = MAILBOX_REQ_CODE;       // Request
        mbox[2] = Tag::GetPhysicalWH as u32; // Display size  // tag
        mbox[3] = 8;       // Buffer size   // val buf size
        mbox[4] = 0;       // Request size  // val size
        mbox[5] = 0;       // Space for horizontal resolution
        mbox[6] = 0;       // Space for vertical resolution
        mbox[7] = Tag::End as u32;       // End tag

        if let None = Mailbox::call(Channel::Tags as u8, &mbox as *const u32 as *const u8) { // @todo virt2phys
            return None
        }
        if mbox[1] != MAILBOX_RESP_CODE_SUCCESS {
            return None
        }
        if mbox[5] == 0 && mbox[6] == 0 { // Qemu emulation returns 0x0
            return Some(Size2d { x: 640, y: 480 })
        }
        Some(Size2d { x: mbox[5], y: mbox[6] })
    }

    fn set_display_size(size: Size2d) -> Option<Display> { // @todo Make Display use VC functions internally instead
        #[repr(align=16)]
        let mut mbox = [0 as u32; 22];
        let mut count: usize = 0;

        count += 1; mbox[count] = MAILBOX_REQ_CODE;       // Request
        count += 1; mbox[count] = Tag::SetPhysicalWH as u32;
        count += 1; mbox[count] = 8;       // Buffer size   // val buf size
        count += 1; mbox[count] = 8;       // Request size  // val size
        count += 1; mbox[count] = size.x;  // Space for horizontal resolution
        count += 1; mbox[count] = size.y;  // Space for vertical resolution
        count += 1; mbox[count] = Tag::SetVirtualWH as u32;
        count += 1; mbox[count] = 8;       // Buffer size   // val buf size
        count += 1; mbox[count] = 8;       // Request size  // val size
        count += 1; mbox[count] = size.x;  // Space for horizontal resolution
        count += 1; mbox[count] = size.y;  // Space for vertical resolution
        count += 1; mbox[count] = Tag::SetDepth as u32;
        count += 1; mbox[count] = 4;       // Buffer size   // val buf size
        count += 1; mbox[count] = 4;       // Request size  // val size
        count += 1; mbox[count] = 16;      // 16 bpp
        count += 1; mbox[count] = Tag::AllocateBuffer as u32;
        count += 1; mbox[count] = 8;       // Buffer size   // val buf size
        count += 1; mbox[count] = 4;       // Request size  // val size
        count += 1; mbox[count] = 16;      // Alignment = 16
        count += 1; mbox[count] = 0;       // Space for response
        count += 1; mbox[count] = Tag::End as u32;
        mbox[0] = (count * 4) as u32;      // Total size

        let max_count = count;

        if let None = Mailbox::call(Channel::Tags as u8, &mbox as *const u32 as *const u8) { // @todo virt2phys
            return None
        }
        if mbox[1] != MAILBOX_RESP_CODE_SUCCESS {
            return None
        }

        count = 2;    /* First tag */
        while mbox[count] != 0 {
            if mbox[count] == Tag::AllocateBuffer as u32 {
                break;
            }

            /* Skip to next tag
             * Advance count by 1 (tag) + 2 (buffer size/value size)
             *                          + specified buffer size
             */
            count += 3 + (mbox[count+1] / 4) as usize;

            if count > max_count {
                loop {}
                return None;
            }
        }

        /* Must be 8 bytes, plus MSB set to indicate a response */
        if mbox[count+2] != 0x80000008 {
            loop {}
            return None;
        }

        /* Framebuffer address/size in response */
        let physical_screenbase = mbox[count+3];
        let screensize = mbox[count+4];

        if physical_screenbase == 0 || screensize == 0 {
            loop {}
            return None
        }

        /* physical_screenbase is the address of the screen in RAM
         * screenbase needs to be the screen address in virtual memory
         */
        // screenbase=mem_p2v(physical_screenbase);
        let screenbase = physical_screenbase;

        /* Get the framebuffer pitch (bytes per line) */
        mbox[0] = 7 * 4;      // Total size
        mbox[1] = 0;      // Request
        mbox[2] = Tag::GetPitch as u32;    // Display size
        mbox[3] = 4;      // Buffer size
        mbox[4] = 0;      // Request size
        mbox[5] = 0;      // Space for pitch
        mbox[6] = Tag::End as u32;

        if let None = Mailbox::call(Channel::Tags as u8, &mbox as *const u32 as *const u8) { // @todo virt2phys
            return None
        }
        if mbox[1] != MAILBOX_RESP_CODE_SUCCESS {
            return None
        }

        /* Must be 4 bytes, plus MSB set to indicate a response */
        if mbox[4] != 0x80000004 {
            loop {}
            return None
        }

        let pitch = mbox[5];
        if pitch == 0 {
            loop {}
            return None
        }

        /* Need to set up max_x/max_y before using Display::write */
        let max_x = size.x / CHARSIZE_X;
        let max_y = size.y / CHARSIZE_Y;

        return Some(Display {
            base: screenbase,
            size: screensize,
            pitch: pitch,
            max_x: max_x,
            max_y: max_y,
        })
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
    loop { unsafe { asm!("wfi"); } }
}
