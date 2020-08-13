use crate::common_items::addr::PhysAddr;
use crate::common_items::size::{Byte, Size};
use uefi::prelude::{Boot, SystemTable};
use uefi::proto::media::file;
use uefi::proto::media::file::File;
use uefi::proto::media::file::FileAttribute;
use uefi::proto::media::file::FileMode;
use uefi::proto::media::fs;
use uefi::table::boot::{AllocateType, MemoryType};
use uefi::ResultExt;

mod kernel_bytes;

struct KernelFileInfo {
    name: &'static str,
    start_address: PhysAddr,
}

impl KernelFileInfo {
    const fn new(name: &'static str, start_address: PhysAddr) -> Self {
        Self {
            name,
            start_address,
        }
    }

    fn get_filename(&self) -> &'static str {
        self.name
    }

    fn address(&self) -> PhysAddr {
        self.start_address
    }
}

// Using the size of binary as the memory consumption is useless because the size of .bss section
// is not included in the binary size. Using ELF file may improve effeciency as it might contain
// the size of memory comsuption.
const KERNEL_FILE: KernelFileInfo = KernelFileInfo::new("kernel.bin", PhysAddr::new(0x200000));

pub fn place_kernel(system_table: &SystemTable<Boot>) -> () {
    let mut root_dir = open_root_dir(system_table);

    open_kernel(system_table, &mut root_dir);
}

fn open_root_dir(system_table: &SystemTable<Boot>) -> file::Directory {
    let simple_file_system = system_table
        .boot_services()
        .locate_protocol::<fs::SimpleFileSystem>()
        .expect_success("Failed to prepare simple file system.");

    let simple_file_system = unsafe { &mut *simple_file_system.get() };

    simple_file_system
        .open_volume()
        .expect_success("Failed to open the root directory.")
}

fn open_kernel(system_table: &SystemTable<Boot>, root_dir: &mut file::Directory) -> () {
    let kernel_bytes = kernel_bytes::get(root_dir);
    let mut kernel_handler = get_kernel_handler(root_dir);

    allocate_for_kernel_file(system_table, kernel_bytes);
    read_kernel_on_memory(&mut kernel_handler, kernel_bytes);
}

fn get_kernel_handler(root_dir: &mut file::Directory) -> file::RegularFile {
    let handler = root_dir
        .open(
            KERNEL_FILE.get_filename(),
            FileMode::Read,
            FileAttribute::empty(),
        )
        .expect_success("Failed to get file handler of the kernel.");

    unsafe { file::RegularFile::new(handler) }
}

fn allocate_for_kernel_file(system_table: &SystemTable<Boot>, kernel_bytes: Size<Byte>) -> () {
    system_table
        .boot_services()
        .allocate_pages(
            AllocateType::Address(KERNEL_FILE.address().as_usize()),
            MemoryType::LOADER_DATA,
            kernel_bytes.as_num_of_pages().as_usize(),
        )
        .expect_success("Failed to allocate memory for the kernel");

    // It is not necessary to return the address as it is fixed.
}

fn read_kernel_on_memory(handler: &mut file::RegularFile, kernel_bytes: Size<Byte>) -> () {
    // Reading should use while statement with the number of bytes which were actually read.
    // However, without while statement previous uefi implementation worked so this uefi
    // implementation also never use it.
    handler
        .read(unsafe {
            core::slice::from_raw_parts_mut(
                KERNEL_FILE.address().as_usize() as *mut u8,
                kernel_bytes.as_usize(),
            )
        })
        .expect_success("Failed to read kernel");
}
