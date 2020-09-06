// SPDX-License-Identifier: GPL-3.0-or-later

use uefi::proto::media::file;
use uefi::proto::media::fs;
use uefi::table::boot;
use uefi::ResultExt;

pub fn open(boot_services: &boot::BootServices) -> file::Directory {
    let simple_file_system = boot_services
        .locate_protocol::<fs::SimpleFileSystem>()
        .expect_success("Failed to prepare simple file system.");

    let simple_file_system = unsafe { &mut *simple_file_system.get() };

    simple_file_system
        .open_volume()
        .expect_success("Failed to open the root directory.")
}
