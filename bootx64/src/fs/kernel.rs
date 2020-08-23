use super::root_dir;
use common_items::constant::KERNEL_NAME;
use common_items::size::{Byte, Size};
use uefi::prelude::{Boot, SystemTable};
use uefi::proto::media::file;
use uefi::proto::media::file::File;
use uefi::proto::media::file::FileAttribute;
use uefi::proto::media::file::FileMode;
use uefi::table::boot::{AllocateType, MemoryType};
use uefi::ResultExt;
use x86_64::PhysAddr;

mod size;

pub fn deploy(system_table: &SystemTable<Boot>) -> (PhysAddr, Size<Byte>) {
    let mut root_dir = root_dir::open(system_table);

    locate(system_table, &mut root_dir)
}

fn locate(
    system_table: &SystemTable<Boot>,
    root_dir: &mut file::Directory,
) -> (PhysAddr, Size<Byte>) {
    let kernel_bytes = size::get(root_dir);
    let mut kernel_handler = get_handler(root_dir);

    let addr = allocate(system_table, kernel_bytes);
    put_on_memory(&mut kernel_handler, addr, kernel_bytes);

    (addr, kernel_bytes)
}

fn get_handler(root_dir: &mut file::Directory) -> file::RegularFile {
    let handler = root_dir
        .open(KERNEL_NAME, FileMode::Read, FileAttribute::empty())
        .expect_success("Failed to get file handler of the kernel.");

    unsafe { file::RegularFile::new(handler) }
}

fn allocate(system_table: &SystemTable<Boot>, kernel_bytes: Size<Byte>) -> PhysAddr {
    PhysAddr::new(
        system_table
            .boot_services()
            .allocate_pages(
                AllocateType::AnyPages,
                MemoryType::LOADER_DATA,
                kernel_bytes.as_num_of_pages().as_usize(),
            )
            .expect_success("Failed to allocate memory for the kernel"),
    )
}

fn put_on_memory(
    handler: &mut file::RegularFile,
    kernel_addr: PhysAddr,
    kernel_bytes: Size<Byte>,
) -> () {
    // Reading should use while statement with the number of bytes which were actually read.
    // However, without while statement previous uefi implementation worked so this uefi
    // implementation also never use it.
    handler
        .read(unsafe {
            core::slice::from_raw_parts_mut(kernel_addr.as_u64() as _, kernel_bytes.as_usize())
        })
        .expect_success("Failed to read kernel");
}
