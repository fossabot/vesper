// See BCM2835-ARM-Peripherals.pdf
// See https://www.raspberrypi.org/forums/viewtopic.php?t=186090 for more details.

// Physical memory is 0x00000000 to 0x40000000
const fn phys2virt(address: u32) -> u32 {
    address // + 0x80000000;
}

// RAM bus address is 0xC0000000 to 0xFFFFFFFF
// Peripheral bus memory is 0x7E000000 to 0x7EFFFFFF
pub fn phys2bus(address: u32) -> u32 {
    address.wrapping_add(0xC0000000) // L2 cache disabled
}

pub fn bus2phys(address: u32) -> u32 {
    address.wrapping_sub(0xC0000000) // L2 cache disabled
}

pub const PERIPHERAL_BASE: u32 = phys2virt(0x3F00_0000); // Base address for all peripherals

