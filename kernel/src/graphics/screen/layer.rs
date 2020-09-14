// SPDX-License-Identifier: GPL-3.0-or-later

use {
    super::Vram,
    alloc::vec::Vec,
    core::{
        convert::TryFrom,
        sync::atomic::{AtomicI32, Ordering::Relaxed},
    },
    rgb::RGB8,
    vek::Vec2,
};

struct LayerCollection(Vec<Layer>);

impl LayerCollection {
    fn add_layer(&mut self, layer: Layer) {
        self.0.push(layer);
    }

    fn repaint(&self) {
        for layer in &self.0 {
            for y in 0..layer.len.y {
                for x in 0..layer.len.x {
                    if let Some(rgb) =
                        layer.buf[usize::try_from(y).unwrap()][usize::try_from(x).unwrap()]
                    {
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
    id: Id,
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
            id: Id::new(),
        }
    }
}

#[derive(Debug)]
struct Id(i32);
impl Id {
    fn new() -> Self {
        static ID: AtomicI32 = AtomicI32::new(0);
        Self(ID.fetch_add(1, Relaxed))
    }
}

#[derive(Debug)]
enum Error {
    NoSuchLayer(Id),
}
