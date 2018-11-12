use arch::*;
use platform::rpi3::{phys2bus, PERIPHERAL_BASE}; // core equivalent of std::ptr::write_volatile

const GPIO_BASE: u32 = PERIPHERAL_BASE + 0x20_0000;

// The offsets for reach register.
// From https://wiki.osdev.org/Raspberry_Pi_Bare_Bones

const GPFSEL0: u32 = GPIO_BASE + 0x00;
const GPFSEL1: u32 = GPIO_BASE + 0x04;
const GPFSEL2: u32 = GPIO_BASE + 0x08;
const GPFSEL3: u32 = GPIO_BASE + 0x0C;
const GPFSEL4: u32 = GPIO_BASE + 0x10;
const GPFSEL5: u32 = GPIO_BASE + 0x14;
const GPSET0: u32 = GPIO_BASE + 0x1C;
const GPSET1: u32 = GPIO_BASE + 0x20;
const GPCLR0: u32 = GPIO_BASE + 0x28;
const GPLEV0: u32 = GPIO_BASE + 0x34;
const GPLEV1: u32 = GPIO_BASE + 0x38;
const GPEDS0: u32 = GPIO_BASE + 0x40;
const GPEDS1: u32 = GPIO_BASE + 0x44;
const GPHEN0: u32 = GPIO_BASE + 0x64;
const GPHEN1: u32 = GPIO_BASE + 0x68;

// Controls actuation of pull up/down to ALL GPIO pins.
const GPPUD: u32 = GPIO_BASE + 0x94;

// Controls actuation of pull up/down for specific GPIO pin.
const GPPUDCLK0: u32 = GPIO_BASE + 0x98;

const GPPUDCLK1: u32 = GPIO_BASE + 0x9C;

// The base address for UART.
const UART0_BASE: u32 = PERIPHERAL_BASE + 0x20_1000;

// The offsets for reach register for the UART.
const UART0_DR: u32 = UART0_BASE + 0x00;
const UART0_RSRECR: u32 = UART0_BASE + 0x04;
const UART0_FR: u32 = UART0_BASE + 0x18;
const UART0_ILPR: u32 = UART0_BASE + 0x20;
const UART0_IBRD: u32 = UART0_BASE + 0x24;
const UART0_FBRD: u32 = UART0_BASE + 0x28;
const UART0_LCRH: u32 = UART0_BASE + 0x2C;
const UART0_CR: u32 = UART0_BASE + 0x30;
const UART0_IFLS: u32 = UART0_BASE + 0x34;
const UART0_IMSC: u32 = UART0_BASE + 0x38;
const UART0_RIS: u32 = UART0_BASE + 0x3C;
const UART0_MIS: u32 = UART0_BASE + 0x40;
const UART0_ICR: u32 = UART0_BASE + 0x44;
const UART0_DMACR: u32 = UART0_BASE + 0x48;
const UART0_ITCR: u32 = UART0_BASE + 0x80;
const UART0_ITIP: u32 = UART0_BASE + 0x84;
const UART0_ITOP: u32 = UART0_BASE + 0x88;
const UART0_TDR: u32 = UART0_BASE + 0x8C;

const UART1_BASE: u32 = PERIPHERAL_BASE + 0x21_5000;

const AUX_IRQ: u32 = UART1_BASE + 0x00;
const AUX_ENABLE: u32 = UART1_BASE + 0x04;
const AUX_MU_IO: u32 = UART1_BASE + 0x40;
const AUX_MU_IER: u32 = UART1_BASE + 0x44;
const AUX_MU_IIR: u32 = UART1_BASE + 0x48;
const AUX_MU_LCR: u32 = UART1_BASE + 0x4C;
const AUX_MU_MCR: u32 = UART1_BASE + 0x50;
const AUX_MU_LSR: u32 = UART1_BASE + 0x54;
const AUX_MU_MSR: u32 = UART1_BASE + 0x58;
const AUX_MU_SCRATCH: u32 = UART1_BASE + 0x5C;
const AUX_MU_CNTL: u32 = UART1_BASE + 0x60;
const AUX_MU_STAT: u32 = UART1_BASE + 0x64;
const AUX_MU_BAUD: u32 = UART1_BASE + 0x68;
