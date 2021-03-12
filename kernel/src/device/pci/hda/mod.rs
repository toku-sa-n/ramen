// SPDX-License-Identifier: GPL-3.0-or-later

mod registers;

use super::config::bar;
use registers::{GlobalControl, Registers};
use x86_64::PhysAddr;

pub(crate) fn main() {
    let a = iter_controller().next();
    let a = match a {
        Some(a) => a,
        None => return,
    };

    let mut r = unsafe { Registers::new(a) };

    r.gctl.update(GlobalControl::clear_controller_reset);
    while !r.gctl.read().get_controller_reset() {}
}

fn iter_controller() -> impl Iterator<Item = PhysAddr> {
    super::iter_devices().filter_map(|c| {
        if c.is_audio_controller() {
            Some(c.base_address(bar::Index::new(0)))
        } else {
            None
        }
    })
}
