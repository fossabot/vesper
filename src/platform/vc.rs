use platform::display::{Display, PixelOrder, Size2d, CHARSIZE_X, CHARSIZE_Y};
use platform::mailbox::{self, channel, Mailbox, tag};
use platform::rpi3::bus2phys;

pub struct VC;

pub enum VcError {
    MboxError(mailbox::MboxError),
    FormatError,
    PixelOrderInvalid,
}

impl VC {
    pub fn init_fb(size: Size2d) -> Result<Display, VcError> {
        // mailbox
        let mut mbox = Mailbox::new();

        // From https://github.com/bztsrc/raspi3-tutorial/blob/master/09_framebuffer/lfb.c
        // @todo macro to `fill_tag(mbox, offset, tag::XX, arg, arg, arg)`
        mbox.buffer[0] = 35 * 4;
        mbox.buffer[1] = mailbox::REQUEST;

        mbox.buffer[2] = tag::SetPhysicalWH;
        mbox.buffer[3] = 8;
        mbox.buffer[4] = 8;
        mbox.buffer[5] = size.x; // GpuFb.width
        mbox.buffer[6] = size.y; // GpuFb.height

        mbox.buffer[7] = tag::SetVirtualWH;
        mbox.buffer[8] = 8;
        mbox.buffer[9] = 8;
        mbox.buffer[10] = size.x; // GpuFb.vwidth
        mbox.buffer[11] = size.y; // GpuFb.vheight

        mbox.buffer[12] = tag::SetVirtualOffset;
        mbox.buffer[13] = 8;
        mbox.buffer[14] = 8;
        mbox.buffer[15] = 0; // GpuFb.x_offset
        mbox.buffer[16] = 0; // GpuFb.y_offset

        mbox.buffer[17] = tag::SetDepth;
        mbox.buffer[18] = 4;
        mbox.buffer[19] = 4;
        mbox.buffer[20] = 24; // GpuFb.depth

        mbox.buffer[21] = tag::SetPixelOrder;
        mbox.buffer[22] = 4;
        mbox.buffer[23] = 4;
        mbox.buffer[24] = PixelOrder::RGB as u32;

        mbox.buffer[25] = tag::AllocateBuffer;
        mbox.buffer[26] = 8;
        mbox.buffer[27] = 8;
        mbox.buffer[28] = 4096; // GpuFb.pointer <- 4K aligned
        mbox.buffer[29] = 0;    // GpuFb.size

        mbox.buffer[30] = tag::GetPitch;
        mbox.buffer[31] = 4;
        mbox.buffer[32] = 4;
        mbox.buffer[33] = 0;    // GpuFb.pitch

        mbox.buffer[34] = tag::End;

        mbox.call(channel::PropertyTagsArmToVc).map_err(VcError::MboxError)?;

        if mbox.buffer[20] != 24 || mbox.buffer[28] == 0 {
            return Err(VcError::FormatError);
        }

        /* Need to set up max_x/max_y before using Display::write */
        let max_x = mbox.buffer[5] / CHARSIZE_X;
        let max_y = mbox.buffer[6] / CHARSIZE_Y;

        let order: PixelOrder = match mbox.buffer[24] {
            0 => PixelOrder::BGR,
            1 => PixelOrder::RGB,
            _ => return Err(VcError::PixelOrderInvalid),
        };

        Ok(Display::new(
            bus2phys(mbox.buffer[28]),
            mbox.buffer[29], // size
            mbox.buffer[33], // pitch
            max_x,
            max_y,
            mbox.buffer[5],
            mbox.buffer[6],
            order,
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
