// SPDX-License-Identifier: GPL-3.0-or-later

use super::Vram;
use crate::{graphics::Vram, mem::allocator::page_box::PageBox};
use conquer_once::spin::OnceCell;
use core::convert::TryInto;
use screen_layer::{Layer, Vec2};
use spinning_top::Spinlock;

pub static CONTROLLER: OnceCell<Spinlock<screen_layer::Controller>> = OnceCell::uninit();

static BUFFER: OnceCell<PageBox<[u8]>> = OnceCell::uninit();

pub fn init() {
    init_buffer();
    init_controller();
}

pub fn add(l: Layer) -> screen_layer::Id {
    get_controller().lock().add_layer(l)
}

pub fn edit<T>(id: screen_layer::Id, f: T) -> Result<(), screen_layer::Error>
where
    T: Fn(&mut Layer),
{
    get_controller().lock().edit_layer(id, f)
}

pub fn slide(id: screen_layer::Id, new_top_left: Vec2<isize>) -> Result<(), screen_layer::Error> {
    get_controller().lock().slide_layer(id, new_top_left)
}

fn init_buffer() {
    BUFFER.init_once(|| PageBox::new_slice(0, Vram::resolution().product() * Vram::bpp() / 8))
}

fn init_controller() {
    CONTROLLER
        .try_init_once(|| {
            Spinlock::new(unsafe {
                screen_layer::Controller::new(
                    Vram::resolution().as_(),
                    Vram::bpp().try_into().unwrap(),
                    buffer_addr().as_u64().try_into().unwrap(),
                )
            })
        })
        .expect("Layer controller is already initialized.")
}

fn buffer_addr() -> VirtAddr {
    BUFFER.try_get().unwrap().virt_addr()
}

fn get_controller() -> &'static Spinlock<screen_layer::Controller> {
    CONTROLLER
        .try_get()
        .expect("Layer controller is not initialized.")
}
