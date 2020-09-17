// SPDX-License-Identifier: GPL-3.0-or-later

use {
    super::Vram,
    conquer_once::spin::OnceCell,
    core::ops::Deref,
    screen_layer::{self, Layer},
    spinning_top::Spinlock,
};

pub static CONTROLLER: OnceCell<Spinlock<screen_layer::Controller>> = OnceCell::uninit();

pub fn init() {
    CONTROLLER
        .try_init_once(|| {
            Spinlock::new(unsafe {
                screen_layer::Controller::new(
                    Vram::resolution().as_(),
                    Vram::bpp() as _,
                    Vram::ptr().as_u64() as _,
                )
            })
        })
        .expect("Layer controller is already initialized.")
}

pub(super) fn get_controller() -> &'static Spinlock<screen_layer::Controller> {
    CONTROLLER
        .try_get()
        .expect("Layer controller is not initialized.")
}
