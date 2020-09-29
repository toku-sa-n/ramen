// SPDX-License-Identifier: GPL-3.0-or-later

pub trait TRB {
    const SIZE: usize = 16;
}

pub struct Command;
impl TRB for Command {}

pub struct Event;
impl TRB for Event {}
