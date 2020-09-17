// SPDX-License-Identifier: GPL-3.0-or-later

use {
    super::{layer, MOUSE_CURSOR_HEIGHT, MOUSE_CURSOR_WIDTH, MOUSE_GRAPHIC},
    crate::graphics::Vram,
    rgb::RGB8,
    screen_layer::{self, Layer},
    vek::Vec2,
};

pub struct Cursor {
    coord: Vec2<i32>,
    id: screen_layer::Id,
}

impl Cursor {
    pub fn new() -> Self {
        let layer = Layer::new(
            Vec2::zero(),
            Vec2::new(MOUSE_CURSOR_WIDTH, MOUSE_CURSOR_HEIGHT).as_(),
        );

        let id = layer::get_controller().lock().add_layer(layer);

        layer::get_controller()
            .lock()
            .edit_layer(id, |layer: &mut Layer| {
                for y in 0..MOUSE_CURSOR_HEIGHT {
                    for x in 0..MOUSE_CURSOR_WIDTH {
                        layer[y][x] = match MOUSE_GRAPHIC[y][x] {
                            '*' => Some(RGB8::new(0, 0, 0)),
                            '0' => Some(RGB8::new(0xff, 0xff, 0xff)),
                            _ => None,
                        }
                    }
                }
            })
            .expect("Layer of mouse cursor should be added.");

        Self {
            coord: Vec2::new(0, 0),
            id,
        }
    }

    pub fn move_offset(&mut self, offset: Vec2<i32>) {
        let new_coord = self.coord + offset;
        self.coord = new_coord;
        self.fit_in_screen();
        layer::get_controller()
            .lock()
            .slide_layer(self.id, self.coord.as_())
            .expect("Layer of mouse cursor should be added.");
    }

    fn fit_in_screen(&mut self) {
        self.coord = Vec2::<i32>::max(
            Vec2::min(self.coord, *Vram::resolution() - Vec2::one()),
            Vec2::zero(),
        );
    }
}
