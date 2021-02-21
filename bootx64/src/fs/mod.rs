// SPDX-License-Identifier: GPL-3.0-or-later

mod root_dir;

use alloc::vec;
use common::constant::KERNEL_ADDR;
use core::{
    convert::{TryFrom, TryInto},
    slice,
};
use elf_rs::Elf;
use file::{FileType, RegularFile};
use os_units::Bytes;
use uefi::{
    proto::media::{
        file,
        file::{File, FileAttribute, FileMode},
    },
    table::{
        boot,
        boot::{AllocateType, MemoryType},
    },
    ResultExt,
};
use x86_64::{structures::paging::Size4KiB, PhysAddr, VirtAddr};

#[must_use]
pub fn deploy(bs: &boot::BootServices, name: &'static str) -> (PhysAddr, Bytes) {
    let mut root_dir = root_dir::open(bs);

    locate(bs, &mut root_dir, name)
}

fn locate(
    bs: &boot::BootServices,
    root: &mut file::Directory,
    name: &'static str,
) -> (PhysAddr, Bytes) {
    let mut file_handler = get_handler(root, name);
    let file_bytes = size(&mut file_handler);

    let addr = allocate(bs, file_bytes);
    put_on_memory(&mut file_handler, addr, file_bytes);

    (addr, file_bytes)
}

/// # Panics
///
/// This function panics if the given file is not an ELF file.
#[must_use]
pub fn fetch_entry_address_and_memory_size(addr: PhysAddr, bytes: Bytes) -> (VirtAddr, Bytes) {
    let elf =
        Elf::from_bytes(unsafe { slice::from_raw_parts(addr.as_u64() as _, bytes.as_usize()) });
    let elf = elf.expect("Failed to get the ELF information.");

    match elf {
        Elf::Elf32(_) => panic!("32-bit kernel is not supported"),
        Elf::Elf64(elf) => {
            let entry_addr = VirtAddr::new(elf.header().entry_point());
            let mem_size = elf.program_header_iter().fold(Bytes::new(0), |acc, x| {
                acc.max(Bytes::new(
                    (x.ph.vaddr() + x.ph.memsz()).try_into().unwrap(),
                ))
            }) - Bytes::new(usize::try_from(KERNEL_ADDR.as_u64()).unwrap());

            info!("Entry point: {:?}", entry_addr);
            info!("Memory size: {:X?}", mem_size.as_usize());

            (entry_addr, mem_size)
        }
    }
}

fn get_handler(root: &mut file::Directory, name: &'static str) -> file::RegularFile {
    let h = root
        .open(name, FileMode::Read, FileAttribute::empty())
        .expect_success("Failed to get file handler of the kernel.");

    let h = h
        .into_type()
        .expect_success("Failed to get the type of a file.");

    match h {
        FileType::Regular(r) => r,
        FileType::Dir(_) => {
            panic!("Not a regular file.")
        }
    }
}

fn allocate(boot_services: &boot::BootServices, kernel_bytes: Bytes) -> PhysAddr {
    PhysAddr::new(
        boot_services
            .allocate_pages(
                AllocateType::AnyPages,
                MemoryType::LOADER_DATA,
                kernel_bytes.as_num_of_pages::<Size4KiB>().as_usize(),
            )
            .expect_success("Failed to allocate memory for the kernel"),
    )
}

fn put_on_memory(handler: &mut file::RegularFile, kernel_addr: PhysAddr, kernel_bytes: Bytes) {
    // Reading should use while statement with the number of bytes which were actually read.
    // However, without while statement previous uefi implementation worked so this uefi
    // implementation also never use it.
    handler
        .read(unsafe {
            core::slice::from_raw_parts_mut(kernel_addr.as_u64() as _, kernel_bytes.as_usize())
        })
        .expect_success("Failed to read kernel");
}

fn size(r: &mut file::RegularFile) -> Bytes {
    // Allocate a too small buffer deliberately to get the number of bytes which is enough for a
    // buffer.
    let mut b = vec![0_u8; 1];
    let (s, bytes) = r
        .get_info::<FileInfo>(&mut b)
        .expect_error("The buffer should be too small.")
        .split();
    assert_eq!(
        s,
        uefi::Status::BUFFER_TOO_SMALL,
        "Unexpected error was returned."
    );
    let bytes = bytes.expect("The number of bytes was not returned.");

    let mut b = vec![0_u8; bytes];
    let i = r
        .get_info::<FileInfo>(&mut b)
        .expect_success("`get_info` failed.");

    Bytes::new(i.file_size().try_into().unwrap())
}
