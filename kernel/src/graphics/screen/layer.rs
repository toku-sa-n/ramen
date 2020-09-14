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
    vek::Vec2,
};

pub static CONTROLLER: OnceCell<Controller> = OnceCell::uninit();

pub struct Controller(Vec<Layer>);

pub fn init_controller() {
    CONTROLLER
        .try_init_once(Controller::new)
        .expect("CONTROLLER is already initialized.")
}

impl Controller {
    fn new() -> Self {
        Self(Vec::new())
    }

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

    fn slide_layer(&mut self, id: Id, new_top_left: Vec2<i32>) -> Result<(), Error> {
        let layer = self.id_to_layer(id)?;
        layer.slide(new_top_left);
        Ok(())
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

#[derive(Debug, PartialEq, Eq)]
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
