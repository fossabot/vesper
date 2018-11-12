use arch::*;
use core::ops;
use platform::rpi3::PERIPHERAL_BASE;
use register::mmio::*;

// Public interface to the mailbox
#[repr(C)]
#[repr(align(16))]
pub struct Mailbox {
    // The address for buffer needs to be 16-byte aligned
    // so that the VideoCore can handle it properly.
    pub buffer: [u32; 36],
}

// Identity mapped first 1Gb by u-boot
const MAILBOX_BASE: u32 = PERIPHERAL_BASE + 0xb880;
/* Lower 4-bits are channel ID */
const CHANNEL_MASK: u32 = 0xf;

// Mailbox Peek  Read/Write  Status  Sender  Config
//    0    0x10  0x00        0x18    0x14    0x1c
//    1    0x30  0x20        0x38    0x34    0x3c
//
// Only mailbox 0's status can trigger interrupts on the ARM, so Mailbox 0 is
// always for communication from VC to ARM and Mailbox 1 is for ARM to VC.
//
// The ARM should never write Mailbox 0 or read Mailbox 1.

// Based on https://github.com/rust-embedded/rust-raspi3-tutorial/blob/master/04_mailboxes/src/mbox.rs
// by Andre Richter of Tock OS.

register_bitfields! {
    u32,

    STATUS [
        /* Bit 31 set in status register if the write mailbox is full */
        FULL  OFFSET(31) NUMBITS(1) [],
        /* Bit 30 set in status register if the read mailbox is empty */
        EMPTY OFFSET(30) NUMBITS(1) []
    ]
}

#[allow(non_snake_case)]
#[repr(C)]
pub struct RegisterBlock {
    READ: ReadOnly<u32>,    // 0x00  This is Mailbox0 read for ARM, can't write
    __reserved_0: [u32; 5], // 0x04
    STATUS: ReadOnly<u32, STATUS::Register>, // 0x18
    __reserved_1: u32,      // 0x1C
    WRITE: WriteOnly<u32>,  // 0x20  This is Mailbox1 write for ARM, can't read
}

pub enum MboxError {
    ResponseError,
    UnknownError,
    Timeout,
}

pub type Result<T> = ::core::result::Result<T, MboxError>;

/*
 * Source https://elinux.org/RPi_Framebuffer
 * Source for channels 8 and 9: https://github.com/raspberrypi/firmware/wiki/Mailboxes
 */
#[allow(non_upper_case_globals)]
pub mod channel {
    pub const Power: u32 = 0;
    pub const Framebuffer: u32 = 1;
    pub const VirtualUart: u32 = 2;
    pub const VChiq: u32 = 3;
    pub const Leds: u32 = 4;
    pub const Buttons: u32 = 5;
    pub const TouchScreen: u32 = 6;
    // Count = 7,
    pub const PropertyTagsArmToVc: u32 = 8;
    pub const PropertyTagsVcToArm: u32 = 9;
}

// Framebuffer channel supported structure (unused)
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

pub const REQUEST: u32 = 0;

// Responses
mod response {
    pub const SUCCESS: u32 = 0x8000_0000;
    pub const ERROR: u32 = 0x8000_0001; // error parsing request buffer (partial response)
}

/* When responding, the VC sets this bit in val_len to indicate a response */
const MAILBOX_TAG_VAL_LEN_RESPONSE: u32 = 0x8000_0000;

#[allow(non_upper_case_globals)]
pub mod tag {
    pub const GetBoardRev: u32 = 0x0001_0002;
    pub const GetMacAddress: u32 = 0x0001_0003;
    pub const GetBoardSerial: u32 = 0x0001_0004;
    pub const GetArmMemory: u32 = 0x0001_0005;
    pub const GetPowerState: u32 = 0x0002_0001;
    pub const SetPowerState: u32 = 0x0002_8001;
    pub const GetClockRate: u32 = 0x0003_0002;
    pub const AllocateBuffer: u32 = 0x0004_0001;
    pub const ReleaseBuffer: u32 = 0x0004_8001;
    pub const BlankScreen: u32 = 0x0004_0002;
    /* Physical means output signal */
    pub const GetPhysicalWH: u32 = 0x0004_0003;
    pub const TestPhysicalWH: u32 = 0x0004_4003;
    pub const SetPhysicalWH: u32 = 0x0004_8003;
    /* Virtual means display buffer */
    pub const GetVirtualWH: u32 = 0x0004_0004;
    pub const TestVirtualWH: u32 = 0x0004_4004;
    pub const SetVirtualWH: u32 = 0x0004_8004;
    pub const GetDepth: u32 = 0x0004_0005;
    pub const TestDepth: u32 = 0x0004_4005;
    pub const SetDepth: u32 = 0x0004_8005;
    pub const GetPixelOrder: u32 = 0x0004_0006;
    pub const TestPixelOrder: u32 = 0x0004_4006;
    pub const SetPixelOrder: u32 = 0x0004_8006;
    pub const GetAlphaMode: u32 = 0x0004_0007;
    pub const TestAlphaMode: u32 = 0x0004_4007;
    pub const SetAlphaMode: u32 = 0x0004_8007;
    pub const GetPitch: u32 = 0x0004_0008;
    /* Offset of display window within buffer */
    pub const GetVirtualOffset: u32 = 0x0004_0009;
    pub const TestVirtualOffset: u32 = 0x0004_4009;
    pub const SetVirtualOffset: u32 = 0x0004_8009;
    pub const GetOverscan: u32 = 0x0004_000a;
    pub const TestOverscan: u32 = 0x0004_400a;
    pub const SetOverscan: u32 = 0x0004_800a;
    pub const GetPalette: u32 = 0x0004_000b;
    pub const TestPalette: u32 = 0x0004_400b;
    pub const SetPalette: u32 = 0x0004_800b;
    pub const End: u32 = 0;
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

/// Deref to RegisterBlock
///
/// Allows writing
/// ```
/// self.STATUS.read()
/// ```
/// instead of something along the lines of
/// ```
/// unsafe { (*Mbox::ptr()).STATUS.read() }
/// ```
impl ops::Deref for Mailbox {
    type Target = RegisterBlock;

    fn deref(&self) -> &Self::Target {
        unsafe { &*Self::ptr() }
    }
}

impl Mailbox {
    pub fn new() -> Mailbox {
        Mailbox { buffer: [0; 36] }
    }

    /// Returns a pointer to the register block
    fn ptr() -> *const RegisterBlock {
        MAILBOX_BASE as *const _
    }

    pub fn write(&self, channel: u32) -> Result<()> {
        let mut count: u32 = 0;

        while self.STATUS.is_set(STATUS::FULL) {
            count += 1;
            if count > (1 << 25) {
                return Err(MboxError::Timeout);
            }
        }
        dmb();
        let buf_ptr = self.buffer.as_ptr() as u32;
        self.WRITE
            .set((buf_ptr & !CHANNEL_MASK) | (channel & CHANNEL_MASK));
        Ok(())
    }

    pub fn read(&self, channel: u32) -> Result<()> {
        let mut count: u32 = 0;

        loop {
            while self.STATUS.is_set(STATUS::EMPTY) {
                count += 1;
                if count > (1 << 25) {
                    return Err(MboxError::Timeout);
                }
            }

            /* Read the data
             * Data memory barriers as we've switched peripheral
             */
            dmb();
            let data: u32 = self.READ.get();
            dmb();

            let buf_ptr = self.buffer.as_ptr() as u32;

            // is it a response to our message?
            if ((data & CHANNEL_MASK) == channel) && ((data & !CHANNEL_MASK) == buf_ptr) {
                // is it a valid successful response?
                return match self.buffer[1] {
                    response::SUCCESS => Ok(()),
                    response::ERROR => Err(MboxError::ResponseError),
                    _ => Err(MboxError::UnknownError),
                };
            }
        }
    }

    pub fn call(&self, channel: u32) -> Result<()> {
        self.write(channel)?;
        self.read(channel)
    }
}
