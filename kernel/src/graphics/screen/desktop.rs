// SPDX-License-Identifier: GPL-3.0-or-later

use {
    super::{
        layer::{self, Layer},
        Screen, Vram,
    },
    core::convert::TryFrom,
    rgb::RGB8,
    vek::Vec2,
};

pub struct Desktop {
    id: layer::Id,
}

impl Desktop {
    pub fn new() -> Self {
        let layer = Layer::new(Vec2::new(0, 0), *Vram::resolution());
        let id = layer::CONTROLLER.lock().add_layer(layer);

        Self { id }
    }

    pub fn draw(&self) {
        let edit = |layer: &mut Layer| {
            let x_len = Vram::resolution().x;
            let y_len = Vram::resolution().y;

            let mut draw_desktop_part = |color, x0, y0, x1, y1| {
                let rgb = RGB8::new(
                    u8::try_from((color >> 16) & 0xff).unwrap(),
                    u8::try_from((color >> 8) & 0xff).unwrap(),
                    u8::try_from(color & 0xff).unwrap(),
                );

                for y in y0..y1 {
                    for x in x0..x1 {
                        layer[usize::try_from(y)
                            .expect("Negative y coordinate during drawing desktop")]
                            [usize::try_from(x)
                                .expect("Negative x coordinate during drawing desktop")] =
                            Some(rgb);
                    }
                }
            };

            draw_desktop_part(0x0000_8484, 0, 0, x_len - 1, y_len - 29);
            draw_desktop_part(0x00C6_C6C6, 0, y_len - 28, x_len - 1, y_len - 28);
            draw_desktop_part(0x00FF_FFFF, 0, y_len - 27, x_len - 1, y_len - 27);
            draw_desktop_part(0x00C6_C6C6, 0, y_len - 26, x_len - 1, y_len - 1);

            draw_desktop_part(0x00FF_FFFF, 3, y_len - 24, 59, y_len - 24);
            draw_desktop_part(0x00FF_FFFF, 2, y_len - 24, 2, y_len - 4);
            draw_desktop_part(0x0084_8484, 3, y_len - 4, 59, y_len - 4);
            draw_desktop_part(0x0084_8484, 59, y_len - 23, 59, y_len - 5);
            draw_desktop_part(0x0000_0000, 2, y_len - 3, 59, y_len - 3);
            draw_desktop_part(0x0000_0000, 60, y_len - 24, 60, y_len - 3);

            draw_desktop_part(0x0084_8484, x_len - 47, y_len - 24, x_len - 4, y_len - 24);
            draw_desktop_part(0x0084_8484, x_len - 47, y_len - 23, x_len - 47, y_len - 4);
            draw_desktop_part(0x00FF_FFFF, x_len - 47, y_len - 3, x_len - 4, y_len - 3);
            draw_desktop_part(0x00FF_FFFF, x_len - 3, y_len - 24, x_len - 3, y_len - 3);
        };

        layer::CONTROLLER
            .lock()
            .edit_layer(self.id, edit)
            .expect("Desktop layer is not added.")
    }
}
