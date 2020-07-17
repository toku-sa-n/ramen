use uefi::prelude::{Boot, SystemTable};
use uefi::proto::media::file;
use uefi::proto::media::fs;
use uefi::ResultExt;

struct KernelFileInfo {
    name: &'static str,
    start_address: usize,
    bytes: usize,
}

impl KernelFileInfo {
    const fn new(name: &'static str, start_address: usize, bytes: usize) -> Self {
        Self {
            name,
            start_address,
            bytes,
        }
    }
}

const KERNEL_FILES: [KernelFileInfo; 2] = [
    KernelFileInfo::new("head.asm.o", 0x500, 0x500),
    KernelFileInfo::new("kernel.bin", 0x200000, 0x20000000),
];

pub fn open_root_dir(system_table: &SystemTable<Boot>) -> file::Directory {
    let simple_file_system = system_table
        .boot_services()
        .locate_protocol::<fs::SimpleFileSystem>()
        .expect_success("Failed to prepare simple file system.");

    let simple_file_system = unsafe { &mut *simple_file_system.get() };

    simple_file_system
        .open_volume()
        .expect_success("Failed to open volume.")
}
