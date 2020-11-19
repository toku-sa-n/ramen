// SPDX-License-Identifier: GPL-3.0-or-later

use super::Vram;
use conquer_once::spin::OnceCell;
use core::convert::TryFrom;
use spinning_top::Spinlock;

pub static CONTROLLER: OnceCell<Spinlock<screen_layer::Controller>> = OnceCell::uninit();

pub fn init() {
    CONTROLLER
        .try_init_once(|| {
            Spinlock::new(unsafe {
                screen_layer::Controller::new(
                    Vram::resolution().as_(),
                    usize::try_from(Vram::bpp()).unwrap(),
                    usize::try_from(Vram::ptr().as_u64()).unwrap(),
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
