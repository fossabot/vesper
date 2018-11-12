use arch::*;
use platform::rpi3::{phys2bus, PERIPHERAL_BASE}; // core equivalent of std::ptr::write_volatile

pub struct Mailbox;

const MAIL_BASE: u32 = PERIPHERAL_BASE + 0xb880;

// Mailbox Peek  Read/Write  Status  Sender  Config
//    0    0x10  0x00        0x18    0x14    0x1c
//    1    0x30  0x20        0x38    0x34    0x3c
//
// Only mailbox 0's status can trigger interrupts on the ARM, so Mailbox 0 is
// always for communication from VC to ARM and Mailbox 1 is for ARM to VC.
//
// The ARM should never write Mailbox 0 or read Mailbox 1.

// Identity mapped first 1Gb by u-boot
const MAILBOX0READ: u32 = MAIL_BASE; // This is Mailbox0 read for ARM, can't write
const MAILBOX0STATUS: u32 = MAIL_BASE + 0x18;
const MAILBOX0WRITE: u32 = MAIL_BASE + 0x20; // This is Mailbox1 write for ARM, can't read

// const MAILBOX_PHYSADDR: u32 = 0x2000b880; // verified: u-boot arch/arm/mach-bcm283x/include/mach/mbox.h

/* Lower 4-bits are channel ID */
const CHANNEL_MASK: u8 = 0xf;

/*
 * Source https://elinux.org/RPi_Framebuffer
 * Source for channels 8 and 9: https://github.com/raspberrypi/firmware/wiki/Mailboxes
 */
#[repr(u8)]
pub enum Channel {
    Power = 0,
    Framebuffer = 1,
    VirtualUart = 2,
    VChiq = 3,
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

// struct MailboxRegs {
//     read: u32,
//     rsvd0: [u32; 5],
//     status: u32,
//     config: u32,
//     write: u32,
// }

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
