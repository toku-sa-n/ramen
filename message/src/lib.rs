// SPDX-License-Identifier: GPL-3.0-or-later

#![no_std]

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Default)]
pub struct Message {
    pub header: Header,
    pub body: Body,
}
impl Message {
    pub fn new(header: Header, body: Body) -> Self {
        Self { header, body }
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Default)]
pub struct Header {
    pub sender: i32,
}
impl Header {
    pub fn new(sender: i32) -> Self {
        Self { sender }
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Default)]
pub struct Body(pub u64, pub u64, pub u64, pub u64, pub u64);
