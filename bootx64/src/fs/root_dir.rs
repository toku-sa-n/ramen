// SPDX-License-Identifier: GPL-3.0-or-later

use uefi::{
    proto::media::{file, fs},
    table::boot,
};

pub(crate) fn open(boot_services: &boot::BootServices) -> file::Directory {
    let handle = boot_services
        .get_handle_for_protocol::<fs::SimpleFileSystem>()
        .expect("Failed to get handle for the simple file system.");

    let mut simple_file_system = boot_services
        .open_protocol_exclusive::<fs::SimpleFileSystem>(handle)
        .expect("Failed to prepare simple file system.");

    simple_file_system
        .open_volume()
        .expect("Failed to open the root directory.")
}
