/// The entry to Rust, all things must be initialized
/// This is called by assembly trampoline, does arch-specific init
/// and passes control to the kernel boot function kmain().
#[no_mangle]
pub unsafe extern fn karch_start() -> ! {
    // Todo: arch-specific init
    ::kmain()
}

// Data memory barrier
pub fn dmb() {
    unsafe { asm!("dmb sy" :::: "volatile"); } // @fixme this is a full barrier
}

pub fn flushcache(address: usize) {
    unsafe { asm!("dc ivac, $0" :: "r"(address) :: "volatile"); }
}

pub fn read_translation_table_base() -> u64 {
    let mut base: u64 = 0;
    unsafe { asm!("mrs $0, ttbr0_el1" : "=r"(base) ::: "volatile"); }
    return base
}

pub fn read_translation_control() -> u64 {
    let mut tcr: u64 = 0;
    unsafe { asm!("mrs $0, tcr_el1" : "=r"(tcr) ::: "volatile"); }
    return tcr
}

pub fn read_mair() -> u64 {
    let mut mair: u64 = 0;
    unsafe { asm!("mrs $0, mair_el1" : "=r"(mair) ::: "volatile"); }
    return mair
}

pub fn write_translation_table_base(base: usize) {
    unsafe { asm!("msr ttbr0_el1, $0" :: "r"(base) :: "volatile"); }
}

// Helper function similar to u-boot
pub fn write_ttbr_tcr_mair(el: u8, base: u64, tcr: u64, attr: u64)
{
    unsafe { asm!("dsb sy" :::: "volatile"); }
    match (el) {
        1 => {
            unsafe { asm!("msr ttbr0_el1, $0
                msr tcr_el1, $1
                msr mair_el1, $2" :: "r"(base), "r"(tcr), "r"(attr) : "memory" : "volatile"); }
        },
        2 => {
            unsafe { asm!("msr ttbr0_el2, $0
                msr tcr_el2, $1
                msr mair_el2, $2" :: "r"(base), "r"(tcr), "r"(attr) : "memory" : "volatile"); }
        },
        3 => {
            unsafe { asm!("msr ttbr0_el3, $0
                msr tcr_el3, $1
                msr mair_el3, $2" :: "r"(base), "r"(tcr), "r"(attr) : "memory" : "volatile"); }
        },
        _ => loop{},
    }
    unsafe { asm!("isb" :::: "volatile"); }
}

pub struct BcmHost;

impl BcmHost {
    // As per https://www.raspberrypi.org/documentation/hardware/raspberrypi/peripheral_addresses.md
    /// This returns the ARM-side physical address where peripherals are mapped.
    pub fn get_peripheral_address() -> usize {
        0x3f000000
    }

    /// This returns the size of the peripheral's space.
    pub fn get_peripheral_size() -> usize {
        0x01000000
    }

    /// This returns the bus address of the SDRAM.
    pub fn get_sdram_address() -> usize {
        0xC0000000 // uncached
    }
}
