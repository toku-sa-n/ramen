pub mod font;

#[macro_use]
pub mod screen;

// Copy trait is needed for constructing MouseCursor struct
// If you are unsure, remove Copy trait from this struct and see the error messages.
#[derive(Clone, Copy)]
pub struct RGB {
    r: u8,
    g: u8,
    b: u8,
}

impl RGB {
    pub fn new(hex: u32) -> Self {
        Self {
            r: ((hex & 0xff0000) >> 16) as u8,
            g: ((hex & 0x00ff00) >> 8) as u8,
            b: (hex & 0x0000ff) as u8,
        }
    }
}

#[derive(Clone)]
pub struct Vram {
    pub bits_per_pixel: i8,
    pub x_len: i16,
    pub y_len: i16,
    pub ptr: *mut u8,
    rgb_table: [[u8; 3]; 16],
}

impl Vram {
    pub fn new() -> Self {
        Self {
            bits_per_pixel: unsafe { *(0x0ff2 as *const i8) },
            x_len: unsafe { *(0x0ff4 as *const i16) },
            y_len: unsafe { *(0x0ff6 as *const i16) },
            ptr: unsafe { &mut *(*(0x0ff8 as *const i32) as *mut u8) },
            rgb_table: [
                [0x00, 0x00, 0x00],
                [0xff, 0x00, 0x00],
                [0x00, 0xff, 0x00],
                [0xff, 0xff, 0x00],
                [0x00, 0x00, 0xff],
                [0xff, 0x00, 0xff],
                [0x00, 0xff, 0xff],
                [0xff, 0xff, 0xff],
                [0xc6, 0xc6, 0xc6],
                [0x84, 0x00, 0x00],
                [0x00, 0x84, 0x00],
                [0x84, 0x84, 0x00],
                [0x00, 0x00, 0x84],
                [0x84, 0x00, 0x84],
                [0x00, 0x84, 0x84],
                [0x84, 0x84, 0x84],
            ],
        }
    }

    pub unsafe fn set_color(&mut self, coord: screen::Coord<isize>, rgb: RGB) -> () {
        let base_ptr: *mut u8 = self
            .ptr
            .offset((coord.y * self.x_len as isize + coord.x) * self.bits_per_pixel as isize / 8);

        // The order of `RGB` is right.
        // See: https://wiki.osdev.org/Drawing_In_Protected_Mode
        *base_ptr.offset(0) = rgb.b;
        *base_ptr.offset(1) = rgb.g;
        *base_ptr.offset(2) = rgb.r;
    }
}
