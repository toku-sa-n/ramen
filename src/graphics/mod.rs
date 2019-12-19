use crate::asm;

pub mod font;

#[macro_use]
pub mod screen;

#[derive(Clone, Copy)]
pub enum ColorIndex {
    Rgb000000 = 0,
    _RgbFF0000 = 1,
    _Rgb00FF00 = 2,
    _RgbFFFF00 = 3,
    _Rgb0000FF = 4,
    _RgbFF00FF = 5,
    _Rgb00FFFF = 6,
    RgbFFFFFF = 7,
    RgbC6C6C6 = 8,
    _Rgb840000 = 9,
    _Rgb008400 = 10,
    _Rgb848400 = 11,
    _Rgb000084 = 12,
    _Rgb840084 = 13,
    Rgb008484 = 14,
    Rgb848484 = 15,
}

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

    pub fn new_from_components(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
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

    fn is_vesa_used(&self) -> bool {
        self.x_len != 320 || self.y_len != 200 || self.bits_per_pixel != 8
    }

    pub fn init_palette(&self) -> () {
        if self.is_vesa_used() {
            return;
        }

        Self::set_palette(0, 15, self.rgb_table);
    }

    fn set_palette(start: i32, end: i32, rgb: [[u8; 3]; 16]) -> () {
        let eflags: i32 = asm::load_eflags();
        asm::cli();
        asm::out8(0x03c8, start);
        for i in start..(end + 1) {
            for j in 0..3 {
                asm::out8(0x03c9, (rgb[i as usize][j] >> 2) as i32);
            }
        }
        asm::store_eflags(eflags);
    }

    pub unsafe fn set_color(&mut self, coord: screen::Coord, rgb: RGB) -> () {
        if self.is_vesa_used() {
            self.set_rgb(coord, rgb);
        } else {
            self.set_closest_color_index(coord, rgb);
        }
    }

    unsafe fn set_rgb(&mut self, coord: screen::Coord, rgb: RGB) -> () {
        let base_ptr: *mut u8 = self
            .ptr
            .offset((coord.y * self.x_len as isize + coord.x) * self.bits_per_pixel as isize / 8);

        // The order of `RGB` is right.
        // See: https://wiki.osdev.org/Drawing_In_Protected_Mode
        *base_ptr.offset(0) = rgb.b;
        *base_ptr.offset(1) = rgb.g;
        *base_ptr.offset(2) = rgb.r;
    }

    unsafe fn set_closest_color_index(&mut self, coord: screen::Coord, rgb: RGB) -> () {
        *self.ptr.offset(coord.y * self.x_len as isize + coord.x) =
            self.calculate_closest_color_index(rgb) as u8;
    }

    fn calculate_closest_color_index(&self, rgb: RGB) -> u8 {
        let mut min_rgb_diff = Self::calculate_rgb_distance(
            &rgb,
            &RGB::new_from_components(
                self.rgb_table[0][0],
                self.rgb_table[0][1],
                self.rgb_table[0][2],
            ),
        );
        let mut best_idx: u8 = 0;

        for i in 0..16 {
            if min_rgb_diff
                > Self::calculate_rgb_distance(
                    &rgb,
                    &RGB::new_from_components(
                        self.rgb_table[i][0],
                        self.rgb_table[i][1],
                        self.rgb_table[i][2],
                    ),
                )
            {
                min_rgb_diff = Self::calculate_rgb_distance(
                    &rgb,
                    &RGB::new_from_components(
                        self.rgb_table[i][0],
                        self.rgb_table[i][1],
                        self.rgb_table[i][2],
                    ),
                );
                best_idx = i as u8;
            }
        }

        best_idx
    }
    fn calculate_rgb_distance(rgb_1: &RGB, rgb_2: &RGB) -> i32 {
        let r_diff: i32 = rgb_1.r as i32 - rgb_2.r as i32;
        let g_diff: i32 = rgb_1.g as i32 - rgb_2.g as i32;
        let b_diff: i32 = rgb_1.b as i32 - rgb_2.b as i32;

        r_diff * r_diff + g_diff * g_diff + b_diff * b_diff
    }
}
