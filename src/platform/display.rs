/* Character cells are 8x8 */
pub const CHARSIZE_X: u32 = 8;
pub const CHARSIZE_Y: u32 = 8;

pub struct Size2d {
    pub x: u32,
    pub y: u32,
}

#[derive(PartialEq)]
pub enum PixelOrder {
    BGR = 0,
    RGB = 1,
}

pub struct Display {
    base: u32,
    size: u32,
    depth: u32,
    pitch: u32,
    max_x: u32,
    max_y: u32,
    width: u32,
    height: u32,
    order: PixelOrder,
}

// https://github.com/david-griffith/rust-bitmap/blob/master/src/lib.rs
#[cfg_attr(rustfmt, rustfmt_skip)]
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

impl core::fmt::Display for Display {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "\n\n\n#### Display(base: {:x}, size: {}, depth: {}, pitch: {}, w: {}, h: {}, order: {})\n\n\n",
            self.base,
            self.size,
            self.depth,
            self.pitch,
            self.width,
            self.height,
            if self.order == PixelOrder::BGR { "BGR" } else { "RGB" }
        )
    }
}

impl Display {
    pub fn new(
        base: u32,
        size: u32,
        depth: u32,
        pitch: u32,
        max_x: u32,
        max_y: u32,
        width: u32,
        height: u32,
        order: PixelOrder,
    ) -> Self {
        Display {
            base,
            size,
            depth,
            pitch,
            max_x,
            max_y,
            width,
            height,
            order,
        }
    }

    pub fn putpixel(&mut self, x: u16, y: u16, color: u32) {
        let c = |chan: u16| {
            if self.order == PixelOrder::BGR {
                2 - chan
            } else {
                chan
            }
        };
        let f = |v: u32, chan: u16| unsafe {
            *(self.base as *mut u8).offset(
                (y as u32 * self.pitch + x as u32 * (self.depth / 8) + c(chan) as u32) as isize,
            ) = v as u8;
        };

        f(color & 0xff, 0);
        f((color >> 8) & 0xff, 1);
        f((color >> 16) & 0xff, 2)
    }

    pub fn rect(&mut self, x1: u16, y1: u16, x2: u16, y2: u16, color: u32) {
        for y in y1..y2 {
            for x in x1..x2 {
                self.putpixel(x, y, color);
            }
        }
    }

    pub fn draw_text(&mut self, x: u16, y: u16, text: &str, color: u32) {
        for i in 0..8 {
            let mut char_off = 0;
            // Take an 8 bit slice from each array value.
            for my_char in text.as_bytes() {
                if (*my_char as isize - 0x20 > 95) || (*my_char as isize - 0x20 < 0) {
                    return; // Err("Character not in font.");
                }

                let mut myval = CHAR_ARRAY[*my_char as usize - 0x20];
                myval = myval.swap_bytes();
                // do initial shr.
                myval = myval >> (i * 8);
                for mycount in 0..8 {
                    if myval & 1 == 1 {
                        self.putpixel(x + char_off * 8 + mycount, y + i, color);
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
}
