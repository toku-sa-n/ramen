// SPDX-License-Identifier: GPL-3.0-or-later

use {super::Vram, alloc::vec::Vec, core::convert::TryFrom, rgb::RGB8, vek::Vec2};

struct LayerCollection(Vec<Layer>);

impl LayerCollection {
    fn add_layer(&mut self, layer: Layer) {
        self.0.push(layer);
    }

    fn repaint(&self) {
        for layer in &self.0 {
            for y in 0..layer.len.y {
                for x in 0..layer.len.x {
                    if let Some(rgb) = layer.buf[y as usize][x as usize] {
                        unsafe { Vram::set_color(layer.top_left + Vec2::new(x, y), rgb) }
                    }
                }
            }
        }
    }
}

struct Layer {
    buf: Vec<Vec<Option<RGB8>>>,
    top_left: Vec2<i32>,
    len: Vec2<i32>,
}

impl Layer {
    fn new(top_left: Vec2<i32>, len: Vec2<i32>) -> Self {
        Self {
            buf: vec![
                vec![None; usize::try_from(len.x).expect("Negative width of a layer.")];
                usize::try_from(len.y).expect("Negative height of a layer.")
            ],
            top_left,
            len,
        }
    }
}
