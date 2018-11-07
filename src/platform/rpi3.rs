// See BCM2835-ARM-Peripherals.pdf
// See https://www.raspberrypi.org/forums/viewtopic.php?t=186090 for more details.

// Physical memory is 0x0000_0000 to 0x4000_0000
const fn phys2virt(address: u32) -> u32 {
    address // + 0x8000_0000;
}

// RAM bus address is 0xC000_0000 to 0xFFFF_FFFF
// Peripheral bus memory is 0x7E00_0000 to 0x7EFF_FFFF
pub fn phys2bus(address: u32) -> u32 {
    address.wrapping_add(0xC000_0000) // L2 cache disabled
}

pub fn bus2phys(address: u32) -> u32 {
    address.wrapping_sub(0xC000_0000) // L2 cache disabled
}

pub const PERIPHERAL_BASE: u32 = phys2virt(0x3F00_0000); // Base address for all peripherals
// @todo BcmHost::get_peripheral_address()
