// SPDX-License-Identifier: GPL-3.0-or-later

use {super::Vram, alloc::vec::Vec, core::convert::TryFrom, rgb::RGB8, vek::Vec2};

struct LayerCollection(Vec<Layer>);

impl LayerCollection {
    fn add_layer(&mut self, layer: Layer) {
        self.0.push(layer);
        self.0.sort_by(|a, b| a.z_index.cmp(&b.z_index));
    }

    fn repaint(&self) {
        for layer in self.0.iter() {
            for y in 0..layer.top_left.y {
                for x in 0..layer.top_left.x {
                    if let Some(rgb) = layer.buf[y][x] {
                        unsafe {
                            Vram::set_color(
                                Vram::resolution().as_() + Vec2::new(x as i32, y as i32),
                                rgb,
                            )
                        }
                    }
                }
            }
        }
    }
}

struct Layer {
    buf: Vec<Vec<Option<RGB8>>>,
    top_left: Vec2<usize>,
    len: Vec2<usize>,
    z_index: usize,
}

impl Layer {
    fn new(top_left: Vec2<usize>, len: Vec2<usize>) -> Self {
        Self {
            buf: Vec::new(),
            top_left,
            len,
            z_index: 0,
        }
    }
}
