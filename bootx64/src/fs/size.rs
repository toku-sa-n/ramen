// SPDX-License-Identifier: GPL-3.0-or-later

use core::convert::TryInto;
use os_units::Bytes;
use uefi::{
    proto::media::{file, file::RegularFile},
    ResultExt,
};

pub fn get(root: &mut file::Directory, name: &'static str) -> Bytes {
    let mut h = super::get_handler(root, name);

    h.set_position(RegularFile::END_OF_FILE)
        .expect_success("Failed to calculate the size of the kernel.");

    let b = h
        .get_position()
        .expect_success("Failed to calculate the size of a binary.");
    Bytes::new(b.try_into().unwrap())
}
