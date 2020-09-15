// SPDX-License-Identifier: GPL-3.0-or-later

use {
    super::Vram,
    alloc::vec::Vec,
    conquer_once::spin::OnceCell,
    core::{
        convert::TryFrom,
        ops::{Index, IndexMut},
        sync::atomic::{AtomicI32, Ordering::Relaxed},
    },
    rgb::RGB8,
    spinning_top::Spinlock,
    vek::Vec2,
};

pub static CONTROLLER: Spinlock<Controller> = Spinlock::new(Controller::new());

pub struct Controller(Vec<Layer>);

impl Controller {
    const fn new() -> Self {
        Self(Vec::new())
    }

    pub fn add_layer(&mut self, layer: Layer) -> Id {
        let id = layer.id;
        self.0.push(layer);
        id
    }

    pub fn edit_layer<T>(&mut self, id: Id, f: T) -> Result<(), Error>
    where
        T: Fn(&mut Layer),
    {
        let layer = self.id_to_layer(id)?;
        let layer_top_left = layer.top_left;
        let layer_len = layer.len;
        f(layer);
        self.repaint(layer_top_left, layer_len);
        Ok(())
    }

    pub fn slide_layer(&mut self, id: Id, new_top_left: Vec2<i32>) -> Result<(), Error> {
        let layer = self.id_to_layer(id)?;
        let old_top_left = layer.top_left;
        let layer_len = layer.len;
        layer.slide(new_top_left);
        self.repaint(old_top_left, layer_len);
        self.repaint(new_top_left, layer_len);
        Ok(())
    }

    fn repaint(&self, vram_top_left: Vec2<i32>, len: Vec2<i32>) {
        for layer in &self.0 {
            for y in layer.top_left.y..(layer.top_left.y + layer.len.y) {
                for x in layer.top_left.x..(layer.top_left.x + layer.len.x) {
                    if let Some(rgb) = layer.buf[usize::try_from(y - layer.top_left.y).unwrap()]
                        [usize::try_from(x - layer.top_left.x).unwrap()]
                    {
                        unsafe { Vram::set_color(Vec2::new(x, y), rgb) }
                    }
                }
            }
        }
    }

    fn bring_layer_to_front(&mut self, id: Id) -> Result<(), Error> {
        let index = self.id_to_index(id)?;

        let layer = self.0.remove(index);
        self.0.push(layer);

        Ok(())
    }

    fn id_to_layer(&mut self, id: Id) -> Result<&mut Layer, Error> {
        self.0
            .iter_mut()
            .find(|layer| layer.id == id)
            .ok_or_else(|| Error::NoSuchLayer(id))
    }

    fn id_to_index(&self, id: Id) -> Result<usize, Error> {
        for (i, layer) in self.0.iter().enumerate() {
            if layer.id == id {
                return Ok(i);
            }
        }

        Err(Error::NoSuchLayer(id))
    }
}

pub struct Layer {
    buf: Vec<Vec<Option<RGB8>>>,
    top_left: Vec2<i32>,
    len: Vec2<i32>,
    id: Id,
}

impl Layer {
    pub fn new(top_left: Vec2<i32>, len: Vec2<i32>) -> Self {
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

    fn slide(&mut self, new_top_left: Vec2<i32>) {
        self.top_left = new_top_left;
    }
}

impl Index<usize> for Layer {
    type Output = Vec<Option<RGB8>>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.buf[index]
    }
}

impl IndexMut<usize> for Layer {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.buf[index]
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct Id(i32);
impl Id {
    fn new() -> Self {
        static ID: AtomicI32 = AtomicI32::new(0);
        Self(ID.fetch_add(1, Relaxed))
    }
}

#[derive(Debug)]
pub enum Error {
    NoSuchLayer(Id),
}
