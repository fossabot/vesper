#![no_std]
#![no_main]
#![feature(asm)]
#![feature(const_fn)]
#![feature(lang_items)]
// #![feature(repr_align)]
#![feature(ptr_internals)] // temp
#![feature(core_intrinsics)]
#![doc(html_root_url = "https://docs.metta.systems/")]

#[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
use architecture_not_supported_sorry;

// use core::intrinsics::abort;
use core::intrinsics::volatile_load; // core equivalent of std::ptr::read_volatile
use core::intrinsics::volatile_store; // core equivalent of std::ptr::write_volatile

#[macro_use]
extern crate bitflags;
extern crate multiboot2;
extern crate rlibc;

use core::panic::PanicInfo;
#[macro_use]
pub mod arch;
pub use arch::*;

// User-facing kernel parts - syscalls and capability invocations.
// pub mod vesper; -- no mod exported, because available through syscall interface

// Actual interfaces to call these syscalls are in vesper-user (similar to libsel4)
// pub mod vesper; -- exported from vesper-user

#[lang = "eh_personality"]
#[no_mangle]
pub extern "C" fn eh_personality() {}

#[lang = "panic_impl"]
#[no_mangle]
pub extern fn panic_impl(_info: &PanicInfo) -> ! {
    // @todo rect() + drawtext("PANIC")?
    loop {}
}

struct Mailbox;

struct VC;

// See BCM2835-ARM-Peripherals.pdf
// See https://www.raspberrypi.org/forums/viewtopic.php?t=186090 for more details.

// Physical memory is 0x00000000 to 0x40000000
const fn phys2virt(address: u32) -> u32 {
    address // + 0x80000000;
}

// RAM bus address is 0xC0000000 to 0xFFFFFFFF
// Peripheral bus memory is 0x7E000000 to 0x7EFFFFFF
fn phys2bus(address: u32) -> u32 {
    address.wrapping_add(0xC0000000) // L2 cache disabled
}

fn bus2phys(address: u32) -> u32 {
    address.wrapping_sub(0xC0000000) // L2 cache disabled
}

const PERIPHERAL_BASE: u32 = phys2virt(0x3F00_0000); // Base address for all peripherals

const MAIL_BASE: u32 = PERIPHERAL_BASE + 0xb880;

// Mailbox Peek  Read/Write  Status  Sender  Config
//    0    0x10  0x00        0x18    0x14    0x1c
//    1    0x30  0x20        0x38    0x34    0x3c
//
// Only mailbox 0's status can trigger interrupts on the ARM, so Mailbox 0 is
// always for communication from VC to ARM and Mailbox 1 is for ARM to VC.
//
// The ARM should never write Mailbox 0 or read Mailbox 1.

// Identity mapped first 1Gb by uboot
const MAILBOX0READ: u32 = MAIL_BASE; // This is Mailbox0 read for ARM, can't write
const MAILBOX0STATUS: u32 = MAIL_BASE + 0x18;
const MAILBOX0WRITE: u32 = MAIL_BASE + 0x20; // This is Mailbox1 write for ARM, can't read

// const MAILBOX_PHYSADDR: u32 = 0x2000b880; // verified: u-boot arch/arm/mach-bcm283x/include/mach/mbox.h

fn mmio_write(reg: u32, val: u32) {
    unsafe { volatile_store(reg as *mut u32, val) }
}

fn mmio_read(reg: u32) -> u32 {
    unsafe { volatile_load(reg as *const u32) }
}

// struct MailboxRegs {
//     read: u32,
//     rsvd0: [u32; 5],
//     status: u32,
//     config: u32,
//     write: u32,
// }

/* Lower 4-bits are channel ID */
const CHANNEL_MASK: u8 = 0xf;

/*
 * Source https://elinux.org/RPi_Framebuffer
 * Source for channels 8 and 9: https://github.com/raspberrypi/firmware/wiki/Mailboxes
 */
#[repr(u8)]
enum Channel {
    Power = 0,
    Framebuffer = 1,
    VirtualUart = 2,
    Vchiq = 3,
    Leds = 4,
    Buttons = 5,
    TouchScreen = 6,
    PropertyTagsArmToVc = 8,
    PropertyTagsVcToArm = 9,
}

const MAILBOX_REQ_CODE: u32 = 0;
const MAILBOX_RESP_CODE_SUCCESS: u32 = 0x8000_0000;

/* When responding, the VC sets this bit in val_len to indicate a response */
const MAILBOX_TAG_VAL_LEN_RESPONSE: u32 = 0x8000_0000;

#[allow(dead_code)]
enum Tag {
    GetBoardRev = 0x0001_0002,
    GetMacAddress = 0x0001_0003,
    GetBoardSerial = 0x0001_0004,
    GetArmMemory = 0x0001_0005,
    GetPowerState = 0x0002_0001,
    SetPowerState = 0x0002_8001,
    GetClockRate = 0x0003_0002,
    AllocateBuffer = 0x0004_0001,
    ReleaseBuffer = 0x0004_8001,
    BlankScreen = 0x0004_0002,
    /* Physical means output signal */
    GetPhysicalWH = 0x0004_0003,
    TestPhysicalWH = 0x0004_4003,
    SetPhysicalWH = 0x0004_8003,
    /* Virtual means display buffer */
    GetVirtualWH = 0x0004_0004,
    TestVirtualWH = 0x0004_4004,
    SetVirtualWH = 0x0004_8004,
    GetDepth = 0x0004_0005,
    TestDepth = 0x0004_4005,
    SetDepth = 0x0004_8005,
    GetPixelOrder = 0x0004_0006,
    TestPixelOrder = 0x0004_4006,
    SetPixelOrder = 0x0004_8006,
    GetAlphaMode = 0x0004_0007,
    TestAlphaMode = 0x0004_4007,
    SetAlphaMode = 0x0004_8007,
    GetPitch = 0x0004_0008,
    /* Offset of display window within buffer */
    GetVirtualOffset = 0x0004_0009,
    TestVirtualOffset = 0x0004_4009,
    SetVirtualOffset = 0x0004_8009,
    GetOverscan = 0x0004_000a,
    TestOverscan = 0x0004_400a,
    SetOverscan = 0x0004_800a,
    GetPalette = 0x0004_000b,
    TestPalette = 0x0004_400b,
    SetPalette = 0x0004_800b,
    End = 0,
}

/*

struct bcm2835_mbox_hdr {
    u32 buf_size;
    u32 code;
};


#define BCM2835_MBOX_INIT_HDR(_m_) { \
        memset((_m_), 0, sizeof(*(_m_))); \
        (_m_)->hdr.buf_size = sizeof(*(_m_)); \
        (_m_)->hdr.code = 0; \
        (_m_)->end_tag = 0; \
    }

/*
 * A message buffer contains a list of tags. Each tag must also start with
 * a standardized header.
 */
struct bcm2835_mbox_tag_hdr {
    u32 tag;
    u32 val_buf_size;
    u32 val_len;
};

#define BCM2835_MBOX_INIT_TAG(_t_, _id_) { \
        (_t_)->tag_hdr.tag = BCM2835_MBOX_TAG_##_id_; \
        (_t_)->tag_hdr.val_buf_size = sizeof((_t_)->body); \
        (_t_)->tag_hdr.val_len = sizeof((_t_)->body.req); \
    }

#define BCM2835_MBOX_INIT_TAG_NO_REQ(_t_, _id_) { \
        (_t_)->tag_hdr.tag = BCM2835_MBOX_TAG_##_id_; \
        (_t_)->tag_hdr.val_buf_size = sizeof((_t_)->body); \
        (_t_)->tag_hdr.val_len = 0; \
    }


/*
 * Below we define the ID and struct for many possible tags. This header only
 * defines individual tag structs, not entire message structs, since in
 * general an arbitrary set of tags may be combined into a single message.
 * Clients of the mbox API are expected to define their own overall message
 * structures by combining the header, a set of tags, and a terminating
 * entry. For example,
 *
 * struct msg {
 *     struct bcm2835_mbox_hdr hdr;
 *     struct bcm2835_mbox_tag_get_arm_mem get_arm_mem;
 *     ... perhaps other tags here ...
 *     u32 end_tag;
 * };
 */


struct bcm2835_mbox_tag_get_board_rev {
    struct bcm2835_mbox_tag_hdr tag_hdr;
    union {
        struct {
        } req;
        struct {
            u32 rev;
        } resp;
    } body;
};


struct bcm2835_mbox_tag_get_mac_address {
    struct bcm2835_mbox_tag_hdr tag_hdr;
    union {
        struct {
        } req;
        struct {
            u8 mac[6];
            u8 pad[2];
        } resp;
    } body;
};


struct bcm2835_mbox_tag_get_board_serial {
    struct bcm2835_mbox_tag_hdr tag_hdr;
    union {
        struct __packed {
            u64 serial;
        } resp;
    } body;
};


struct bcm2835_mbox_tag_get_arm_mem {
    struct bcm2835_mbox_tag_hdr tag_hdr;
    union {
        struct {
        } req;
        struct {
            u32 mem_base;
            u32 mem_size;
        } resp;
    } body;
};

#define BCM2835_MBOX_POWER_DEVID_SDHCI      0
#define BCM2835_MBOX_POWER_DEVID_UART0      1
#define BCM2835_MBOX_POWER_DEVID_UART1      2
#define BCM2835_MBOX_POWER_DEVID_USB_HCD    3
#define BCM2835_MBOX_POWER_DEVID_I2C0       4
#define BCM2835_MBOX_POWER_DEVID_I2C1       5
#define BCM2835_MBOX_POWER_DEVID_I2C2       6
#define BCM2835_MBOX_POWER_DEVID_SPI        7
#define BCM2835_MBOX_POWER_DEVID_CCP2TX     8

#define BCM2835_MBOX_POWER_STATE_RESP_ON    (1 << 0)
/* Device doesn't exist */
#define BCM2835_MBOX_POWER_STATE_RESP_NODEV (1 << 1)


struct bcm2835_mbox_tag_get_power_state {
    struct bcm2835_mbox_tag_hdr tag_hdr;
    union {
        struct {
            u32 device_id;
        } req;
        struct {
            u32 device_id;
            u32 state;
        } resp;
    } body;
};


#define BCM2835_MBOX_SET_POWER_STATE_REQ_ON (1 << 0)
#define BCM2835_MBOX_SET_POWER_STATE_REQ_WAIT   (1 << 1)

struct bcm2835_mbox_tag_set_power_state {
    struct bcm2835_mbox_tag_hdr tag_hdr;
    union {
        struct {
            u32 device_id;
            u32 state;
        } req;
        struct {
            u32 device_id;
            u32 state;
        } resp;
    } body;
};


#define BCM2835_MBOX_CLOCK_ID_EMMC  1
#define BCM2835_MBOX_CLOCK_ID_UART  2
#define BCM2835_MBOX_CLOCK_ID_ARM   3
#define BCM2835_MBOX_CLOCK_ID_CORE  4
#define BCM2835_MBOX_CLOCK_ID_V3D   5
#define BCM2835_MBOX_CLOCK_ID_H264  6
#define BCM2835_MBOX_CLOCK_ID_ISP   7
#define BCM2835_MBOX_CLOCK_ID_SDRAM 8
#define BCM2835_MBOX_CLOCK_ID_PIXEL 9
#define BCM2835_MBOX_CLOCK_ID_PWM   10

struct bcm2835_mbox_tag_get_clock_rate {
    struct bcm2835_mbox_tag_hdr tag_hdr;
    union {
        struct {
            u32 clock_id;
        } req;
        struct {
            u32 clock_id;
            u32 rate_hz;
        } resp;
    } body;
};


struct bcm2835_mbox_tag_allocate_buffer {
    struct bcm2835_mbox_tag_hdr tag_hdr;
    union {
        struct {
            u32 alignment;
        } req;
        struct {
            u32 fb_address;
            u32 fb_size;
        } resp;
    } body;
};


struct bcm2835_mbox_tag_release_buffer {
    struct bcm2835_mbox_tag_hdr tag_hdr;
    union {
        struct {
        } req;
        struct {
        } resp;
    } body;
};


struct bcm2835_mbox_tag_blank_screen {
    struct bcm2835_mbox_tag_hdr tag_hdr;
    union {
        struct {
            /* bit 0 means on, other bots reserved */
            u32 state;
        } req;
        struct {
            u32 state;
        } resp;
    } body;
};


struct bcm2835_mbox_tag_physical_w_h {
    struct bcm2835_mbox_tag_hdr tag_hdr;
    union {
        /* req not used for get */
        struct {
            u32 width;
            u32 height;
        } req;
        struct {
            u32 width;
            u32 height;
        } resp;
    } body;
};


struct bcm2835_mbox_tag_virtual_w_h {
    struct bcm2835_mbox_tag_hdr tag_hdr;
    union {
        /* req not used for get */
        struct {
            u32 width;
            u32 height;
        } req;
        struct {
            u32 width;
            u32 height;
        } resp;
    } body;
};


struct bcm2835_mbox_tag_depth {
    struct bcm2835_mbox_tag_hdr tag_hdr;
    union {
        /* req not used for get */
        struct {
            u32 bpp;
        } req;
        struct {
            u32 bpp;
        } resp;
    } body;
};


#define BCM2835_MBOX_PIXEL_ORDER_BGR        0
#define BCM2835_MBOX_PIXEL_ORDER_RGB        1

struct bcm2835_mbox_tag_pixel_order {
    struct bcm2835_mbox_tag_hdr tag_hdr;
    union {
        /* req not used for get */
        struct {
            u32 order;
        } req;
        struct {
            u32 order;
        } resp;
    } body;
};


#define BCM2835_MBOX_ALPHA_MODE_0_OPAQUE    0
#define BCM2835_MBOX_ALPHA_MODE_0_TRANSPARENT   1
#define BCM2835_MBOX_ALPHA_MODE_IGNORED     2

struct bcm2835_mbox_tag_alpha_mode {
    struct bcm2835_mbox_tag_hdr tag_hdr;
    union {
        /* req not used for get */
        struct {
            u32 alpha;
        } req;
        struct {
            u32 alpha;
        } resp;
    } body;
};


struct bcm2835_mbox_tag_pitch {
    struct bcm2835_mbox_tag_hdr tag_hdr;
    union {
        struct {
        } req;
        struct {
            u32 pitch;
        } resp;
    } body;
};


struct bcm2835_mbox_tag_virtual_offset {
    struct bcm2835_mbox_tag_hdr tag_hdr;
    union {
        /* req not used for get */
        struct {
            u32 x;
            u32 y;
        } req;
        struct {
            u32 x;
            u32 y;
        } resp;
    } body;
};


struct bcm2835_mbox_tag_overscan {
    struct bcm2835_mbox_tag_hdr tag_hdr;
    union {
        /* req not used for get */
        struct {
            u32 top;
            u32 bottom;
            u32 left;
            u32 right;
        } req;
        struct {
            u32 top;
            u32 bottom;
            u32 left;
            u32 right;
        } resp;
    } body;
};


struct bcm2835_mbox_tag_get_palette {
    struct bcm2835_mbox_tag_hdr tag_hdr;
    union {
        struct {
        } req;
        struct {
            u32 data[1024];
        } resp;
    } body;
};


struct bcm2835_mbox_tag_test_palette {
    struct bcm2835_mbox_tag_hdr tag_hdr;
    union {
        struct {
            u32 offset;
            u32 num_entries;
            u32 data[256];
        } req;
        struct {
            u32 is_invalid;
        } resp;
    } body;
};


struct bcm2835_mbox_tag_set_palette {
    struct bcm2835_mbox_tag_hdr tag_hdr;
    union {
        struct {
            u32 offset;
            u32 num_entries;
            u32 data[256];
        } req;
        struct {
            u32 is_invalid;
        } resp;
    } body;
};

/*
 * Pass a raw u32 message to the VC, and receive a raw u32 back.
 *
 * Returns 0 for success, any other value for error.
 */
int bcm2835_mbox_call_raw(u32 chan, u32 send, u32 *recv);

/*
 * Pass a complete property-style buffer to the VC, and wait until it has
 * been processed.
 *
 * This function expects a pointer to the mbox_hdr structure in an attempt
 * to ensure some degree of type safety. However, some number of tags and
 * a termination value are expected to immediately follow the header in
 * memory, as required by the property protocol.
 *
 * Each struct bcm2835_mbox_hdr passed must be allocated with
 * ALLOC_CACHE_ALIGN_BUFFER(x, y, z) to ensure proper cache flush/invalidate.
 *
 * Returns 0 for success, any other value for error.
 */
int bcm2835_mbox_call_prop(u32 chan, struct bcm2835_mbox_hdr *buffer);

 */

/* Bit 31 set in status register if the write mailbox is full */
const MAILBOX_STATUS_WR_FULL: u32 = 0x8000_0000;

/* Bit 30 set in status register if the read mailbox is empty */
const MAILBOX_STATUS_RD_EMPTY: u32 = 0x4000_0000;

impl Mailbox {
    pub fn write(channel: u8, physical_base: *const u8) {
        // let mailbox = MAILBOX_PHYSADDR as *mut MailboxRegs;
        let mut count: u32 = 0;

        while mmio_read(MAILBOX0STATUS) & MAILBOX_STATUS_WR_FULL != 0 {
            flushcache(MAILBOX0STATUS as usize);
            count += 1;
            if count > (1 << 25) {
                return;
            }
        }
        dmb();
        mmio_write(
            MAILBOX0WRITE,
            phys2bus(physical_base as u32 & 0xFFFF_FFF0) | u32::from(channel),
        );
    }

    pub fn read(channel: u8) -> Option<u32> {
        // let mailbox = MAILBOX_PHYSADDR as *mut MailboxRegs;
        let mut count: u32 = 0;

        loop {
            while mmio_read(MAILBOX0STATUS) & MAILBOX_STATUS_RD_EMPTY != 0 {
                flushcache(MAILBOX0STATUS as usize);
                count += 1;
                if count > (1 << 25) {
                    return None;
                }
            }

            /* Read the data
             * Data memory barriers as we've switched peripheral
             */
            dmb();
            let data = mmio_read(MAILBOX0READ);
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

/* Character cells are 8x8 */
const CHARSIZE_X: u32 = 8;
const CHARSIZE_Y: u32 = 8;

struct Display {
    base: u32,
    size: u32,
    pitch: u32,
    max_x: u32,
    max_y: u32,
    width: u32,
    height: u32,
}

// bufsize
// code
// ....
// end tag

// tag code
// val bufsize
// val size
// ...data buf

#[repr(align(16))]
struct Mbox([u32; 22]);

impl Mbox {
    fn new() -> Mbox {
        Mbox { 0: [0; 22] }
    }
}

struct RGB(u32);

impl RGB {
    fn rgb(r: u8, g: u8, b: u8) -> RGB {
        RGB((b as u32) << 16 | (g as u32) << 8 | r as u32)
    }
}

#[repr(align(16))]
struct GpuFb {
    width: u32,
    height: u32,
    vwidth: u32,
    vheight: u32,
    pitch: u32,
    depth: u32,
    x_offset: u32,
    y_offset: u32,
    pointer: u32,
    size: u32,
}

impl GpuFb {
    fn init(size: Size2d) -> GpuFb {
        GpuFb {
            width: size.x,
            height: size.y,
            vwidth: size.x,
            vheight: size.y,
            pitch: 0u32,
            depth: 24u32,
            x_offset: 0u32,
            y_offset: 0u32,
            pointer: 0u32,
            size: 0u32,
        }
    }
}

impl VC {
    // Use mailbox framebuffer interface to initialize
    fn init_fb(size: Size2d) -> Option<Display> {
        /* Need to set up max_x/max_y before using Display::write */
        let max_x = size.x / CHARSIZE_X;
        let max_y = size.y / CHARSIZE_Y;

        let mut fb_info: GpuFb = GpuFb::init(size);

        Mailbox::call(Channel::Framebuffer as u8, &fb_info.width as *const u32 as *const u8)?;

        Some(Display {
            base: bus2phys(fb_info.pointer),
            size: fb_info.size,
            pitch: fb_info.pitch,
            max_x: max_x,
            max_y: max_y,
            width: fb_info.width,
            height: fb_info.height,
        })
    }
/*
    fn get_display_size() -> Option<Size2d> {
        let mut mbox = Mbox::new();

        mbox.0[0] = 8 * 4; // Total size
        mbox.0[1] = MAILBOX_REQ_CODE; // Request
        mbox.0[2] = Tag::GetPhysicalWH as u32; // Display size  // tag
        mbox.0[3] = 8; // Buffer size   // val buf size
        mbox.0[4] = 0; // Request size  // val size
        mbox.0[5] = 0; // Space for horizontal resolution
        mbox.0[6] = 0; // Space for vertical resolution
        mbox.0[7] = Tag::End as u32; // End tag

        Mailbox::call(Channel::PropertyTagsArmToVc as u8, &mbox.0 as *const u32 as *const u8)?;

        if mbox.0[1] != MAILBOX_RESP_CODE_SUCCESS {
            return None;
        }
        if mbox.0[5] == 0 && mbox.0[6] == 0 {
            // Qemu emulation returns 0x0
            return Some(Size2d { x: 640, y: 480 });
        }
        Some(Size2d {
            x: mbox.0[5],
            y: mbox.0[6],
        })
    }

    fn set_display_size(size: Size2d) -> Option<Display> {
        // @todo Make Display use VC functions internally instead
        let mut mbox = Mbox::new();
        let mut count: usize = 0;

        count += 1;
        mbox.0[count] = MAILBOX_REQ_CODE; // Request
        count += 1;
        mbox.0[count] = Tag::SetPhysicalWH as u32;
        count += 1;
        mbox.0[count] = 8; // Buffer size   // val buf size
        count += 1;
        mbox.0[count] = 8; // Request size  // val size
        count += 1;
        mbox.0[count] = size.x; // Space for horizontal resolution
        count += 1;
        mbox.0[count] = size.y; // Space for vertical resolution
        count += 1;
        mbox.0[count] = Tag::SetVirtualWH as u32;
        count += 1;
        mbox.0[count] = 8; // Buffer size   // val buf size
        count += 1;
        mbox.0[count] = 8; // Request size  // val size
        count += 1;
        mbox.0[count] = size.x; // Space for horizontal resolution
        count += 1;
        mbox.0[count] = size.y; // Space for vertical resolution
        count += 1;
        mbox.0[count] = Tag::SetDepth as u32;
        count += 1;
        mbox.0[count] = 4; // Buffer size   // val buf size
        count += 1;
        mbox.0[count] = 4; // Request size  // val size
        count += 1;
        mbox.0[count] = 16; // 16 bpp
        count += 1;
        mbox.0[count] = Tag::AllocateBuffer as u32;
        count += 1;
        mbox.0[count] = 8; // Buffer size   // val buf size
        count += 1;
        mbox.0[count] = 4; // Request size  // val size
        count += 1;
        mbox.0[count] = 16; // Alignment = 16
        count += 1;
        mbox.0[count] = 0; // Space for response
        count += 1;
        mbox.0[count] = Tag::End as u32;
        mbox.0[0] = (count * 4) as u32; // Total size

        let max_count = count;

        Mailbox::call(Channel::PropertyTagsArmToVc as u8, &mbox.0 as *const u32 as *const u8)?;

        if mbox.0[1] != MAILBOX_RESP_CODE_SUCCESS {
            return None;
        }

        count = 2; /* First tag */
        while mbox.0[count] != 0 {
            if mbox.0[count] == Tag::AllocateBuffer as u32 {
                break;
            }

            /* Skip to next tag
             * Advance count by 1 (tag) + 2 (buffer size/value size)
             *                          + specified buffer size
             */
            count += 3 + (mbox.0[count + 1] / 4) as usize;

            if count > max_count {
                return None;
            }
        }

        /* Must be 8 bytes, plus MSB set to indicate a response */
        if mbox.0[count + 2] != 0x8000_0008 {
            return None;
        }

        /* Framebuffer address/size in response */
        let physical_screenbase = mbox.0[count + 3];
        let screensize = mbox.0[count + 4];

        if physical_screenbase == 0 || screensize == 0 {
            return None;
        }

        /* physical_screenbase is the address of the screen in RAM
         * screenbase needs to be the screen address in virtual memory
         */
        // screenbase=mem_p2v(physical_screenbase);
        let screenbase = physical_screenbase;

        /* Get the framebuffer pitch (bytes per line) */
        mbox.0[0] = 7 * 4; // Total size
        mbox.0[1] = 0; // Request
        mbox.0[2] = Tag::GetPitch as u32; // Display size
        mbox.0[3] = 4; // Buffer size
        mbox.0[4] = 0; // Request size
        mbox.0[5] = 0; // Space for pitch
        mbox.0[6] = Tag::End as u32;

        Mailbox::call(Channel::PropertyTagsArmToVc as u8, &mbox.0 as *const u32 as *const u8)?;

        if mbox.0[1] != MAILBOX_RESP_CODE_SUCCESS {
            return None;
        }

        /* Must be 4 bytes, plus MSB set to indicate a response */
        if mbox.0[4] != 0x8000_0004 {
            return None;
        }

        let pitch = mbox.0[5];
        if pitch == 0 {
            return None;
        }

        /* Need to set up max_x/max_y before using Display::write */
        let max_x = size.x / CHARSIZE_X;
        let max_y = size.y / CHARSIZE_Y;

        Some(Display {
            base: screenbase,
            size: screensize,
            pitch: pitch,
            max_x: max_x,
            max_y: max_y,
        })
    }*/
}

fn putpixel(x: u16, y: u16, color: u32, display: &mut Display) {
    let f = |v: u32, chan: u16| {
        unsafe { *(display.base as *mut u8).offset((y as u32 * display.pitch + x as u32 * 3 + chan as u32) as isize) = v as u8; }
    };

    f(color & 0xff, 0);
    f((color >> 8) & 0xff, 1);
    f((color >> 16) & 0xff, 2)
}

fn rect(x1: u16, y1: u16, x2: u16, y2: u16, color: u32, display: &mut Display) {
    for y in y1..y2 {
        for x in x1..x2 {
            putpixel(x, y, color, display);
        }
    }
}

// https://github.com/david-griffith/rust-bitmap/blob/master/src/lib.rs
static CHAR_ARRAY: [u64; 95] = [
    0x0000000000000000,                                             // space
    0x183c3c1818001800, 0x3636000000000000, 0x36367f367f363600,     // ! " #
    0x0c3e031e301f0c00, 0x006333180c666300, 0x1c361c6e3b336e00,     // $ % &
    0x0606030000000000, 0x180c0606060c1800, 0x060c1818180c0600,     // ' ( )
    0x00663cff3c660000, 0x000c0c3f0c0c0000, 0x00000000000c0c06,     // * + ,
    0x0000003f00000000, 0x00000000000c0c00, 0x6030180c06030100,     // - . /
    0x3e63737b6f673e00, 0x0c0e0c0c0c0c3f00, 0x1e33301c06333f00,     // 0 1 2
    0x1e33301c30331e00, 0x383c36337f307800, 0x3f031f3030331e00,     // 3 4 5
    0x1c06031f33331e00, 0x3f3330180c0c0c00, 0x1e33331e33331e00,     // 6 7 8
    0x1e33333e30180e00, 0x000c0c00000c0c00, 0x000c0c00000c0c06,     // 9 : ;
    0x180c0603060c1800, 0x00003f00003f0000, 0x060c1830180c0600,     // < = >
    0x1e3330180c000c00, 0x3e637b7b7b031e00, 0x0c1e33333f333300,     // ? @ A
    0x3f66663e66663f00, 0x3c66030303663c00, 0x1f36666666361f00,     // B C D
    0x7f46161e16467f00, 0x7f46161e16060f00, 0x3c66030373667c00,     // E F G
    0x3333333f33333300, 0x1e0c0c0c0c0c1e00, 0x7830303033331e00,     // H I J
    0x6766361e36666700, 0x0f06060646667f00, 0x63777f7f6b636300,     // K L M
    0x63676f7b73636300, 0x1c36636363361c00, 0x3f66663e06060f00,     // N O P
    0x1e3333333b1e3800, 0x3f66663e36666700, 0x1e33070e38331e00,     // Q R S
    0x3f2d0c0c0c0c1e00, 0x3333333333333f00, 0x33333333331e0c00,     // T U V
    0x6363636b7f776300, 0x6363361c1c366300, 0x3333331e0c0c1e00,     // W X Y
    0x7f6331184c667f00, 0x1e06060606061e00, 0x03060c1830604000,     // Z [ \
    0x1e18181818181e00, 0x081c366300000000, 0x00000000000000ff,     // ] ^ _
    0x0c0c180000000000, 0x00001e303e336e00, 0x0706063e66663b00,     // ` a b
    0x00001e3303331e00, 0x3830303e33336e00, 0x00001e333f031e00,     // c d e
    0x1c36060f06060f00, 0x00006e33333e301f, 0x0706366e66666700,     // f g h
    0x0c000e0c0c0c1e00, 0x300030303033331e, 0x070666361e366700,     // i j k
    0x0e0c0c0c0c0c1e00, 0x0000337f7f6b6300, 0x00001f3333333300,     // l m n
    0x00001e3333331e00, 0x00003b66663e060f, 0x00006e33333e3078,     // o p q
    0x00003b6e66060f00, 0x00003e031e301f00, 0x080c3e0c0c2c1800,     // r s t
    0x0000333333336e00, 0x00003333331e0c00, 0x0000636b7f7f3600,     // u v w
    0x000063361c366300, 0x00003333333e301f, 0x00003f190c263f00,     // x y z
    0x380c0c070c0c3800, 0x1818180018181800, 0x070c0c380c0c0700,     // { | }
    0x6e3b000000000000,                                             // ~
];

fn draw_text(x: u16, y: u16, text: &str, color: u32, display: &mut Display) {
    for i in 0..8 {
        let mut char_off = 0;
        // Take an 8 bit slice from each array value.
        for my_char in text.as_bytes() {
            if (*my_char as isize - 0x20 > 95) || (*my_char as isize - 0x20 < 0) {
                return;// Err("Character not in font.");
            }

            let mut myval = CHAR_ARRAY[*my_char as usize - 0x20];
            myval = myval.swap_bytes();
            // do initial shr.
            myval = myval >> (i * 8);
            for mycount in 0..8 {
                if myval & 1 == 1 {
                    putpixel(x + char_off * 8 + mycount, y + i, color, display);
                }
                myval = myval >> 1;
                if myval == 0 {
                    break;
                }
            }
            char_off += 1;
        }
    }
}

// Kernel entry point
// arch crate is responsible for calling this
pub fn kmain() -> ! {
    if let Some(mut display) = VC::init_fb(Size2d { x: 800, y: 600 }) {
        rect(100, 100, 200, 200, RGB::rgb(255,255,255).0, &mut display);
        draw_text(50, 50, "Hello world!", RGB::rgb(0,0,255).0, &mut display);
        // draw_text(50, 150, core::fmt("Display width {}", display.width), RGB::rgb(255,0,0).0, &mut display);
    }

    loop {}
}
