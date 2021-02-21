// SPDX-License-Identifier: GPL-3.0-or-later

use core::convert::TryFrom;
use os_units::Bytes;
use uefi::{
    proto::media::{file, file::RegularFile},
    ResultExt,
};

pub fn get(root_dir: &mut file::Directory, name: &'static str) -> Bytes {
    let mut handler = super::get_handler(root_dir, name);

    handler
        .set_position(RegularFile::END_OF_FILE)
        .expect_success("Failed to calculate the size of the kernel.");

    Bytes::new(
        usize::try_from(
            handler
                .get_position()
                .expect("Failed to calculate the size of the kernel.")
                .unwrap(),
        )
        .unwrap(),
    )
}
