use platform::display::{Display, PixelOrder, Size2d, CHARSIZE_X, CHARSIZE_Y};
use platform::mailbox::{Channel, Mailbox, Tag, MAILBOX_REQ_CODE};
use platform::rpi3::bus2phys;

// bufsize
// code
// ....
// end tag

// tag code
// val bufsize
// val size
// ...data buf

//#[repr(align(16))]
//struct Mbox([u32; 22]);
//
//impl Mbox {
//    fn new() -> Mbox {
//        Mbox { 0: [0; 22] }
//    }
//}

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

#[repr(align(16))]
struct SetPixelOrder {
    total: u32,
    req: u32,
    tag: u32,
    bufsz: u32,
    reqsz: u32,
    param: u32,
}

impl SetPixelOrder {
    fn init() -> Self {
        SetPixelOrder {
            total: 24u32,
            req: MAILBOX_REQ_CODE,
            tag: Tag::SetPixelOrder as u32,
            bufsz: 4,
            reqsz: 4,
            param: 0, // 0 - BGR, 1 - RGB
        }
    }
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

pub struct VC;

impl VC {
    // Use mailbox framebuffer interface to initialize
    pub fn init_fb(size: Size2d) -> Option<Display> {
        /* Need to set up max_x/max_y before using Display::write */
        let max_x = size.x / CHARSIZE_X;
        let max_y = size.y / CHARSIZE_Y;

        let fb_info: GpuFb = GpuFb::init(size);

        Mailbox::call(
            Channel::Framebuffer as u8,
            &fb_info.width as *const u32 as *const u8,
        )?;

        let pixel_order = SetPixelOrder::init();

        Mailbox::call(
            Channel::PropertyTagsArmToVc as u8,
            &pixel_order.total as *const u32 as *const u8,
        )?;

        Some(Display::new(
            bus2phys(fb_info.pointer),
            fb_info.size,
            fb_info.pitch,
            max_x,
            max_y,
            fb_info.width,
            fb_info.height,
            PixelOrder::BGR,
        ))
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
