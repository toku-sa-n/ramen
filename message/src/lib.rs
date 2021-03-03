// SPDX-License-Identifier: GPL-3.0-or-later

#![no_std]

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Default)]
pub struct Message {
    pub m1: u64,
    pub m2: u64,
    pub m3: u64,
    pub m4: u64,
}
impl Message {
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(m1: u64, m2: u64, m3: u64, m4: u64) -> Self {
        Self { m1, m2, m3, m4 }
    }
}
