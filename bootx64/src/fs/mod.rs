// SPDX-License-Identifier: GPL-3.0-or-later

mod root_dir;

use {
    core::{
        convert::{TryFrom, TryInto},
        slice,
    },
    elf_rs::Elf,
    file::{FileInfo, FileType},
    log::info,
    os_units::Bytes,
    uefi::{
        proto::media::{
            file,
            file::{File, FileAttribute, FileMode},
        },
        table::{
            boot,
            boot::{AllocateType, MemoryType},
        },
        ResultExt,
    },
    x86_64::{structures::paging::Size4KiB, PhysAddr, VirtAddr},
};

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
    let file_bytes = size(bs, &mut file_handler);

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
            }) - Bytes::new(
                usize::try_from(predefined_mmap::kernel().start.start_address().as_u64()).unwrap(),
            );

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

fn size(bs: &boot::BootServices, r: &mut file::RegularFile) -> Bytes {
    let info_bytes = bytes_for_get_info(r);

    let n = info_bytes.as_num_of_pages::<Size4KiB>().as_usize();
    let buf = bs.allocate_pages(AllocateType::AnyPages, MemoryType::LOADER_DATA, n);
    let buf = buf.expect_success("Failed to allocate memory for getting the size of a file.");
    let s = unsafe { slice::from_raw_parts_mut(buf as *mut u8, info_bytes.as_usize()) };

    let i = r
        .get_info::<FileInfo>(s)
        .expect_success("`get_info` failed.");

    let sz = Bytes::new(i.file_size().try_into().unwrap());
    bs.free_pages(buf, n)
        .expect_success("Failed to free memory.");
    sz
}

fn bytes_for_get_info(r: &mut file::RegularFile) -> Bytes {
    let (s, bytes) = r
        .get_info::<FileInfo>(&mut [0_u8])
        .expect_error("The buffer should be too small.")
        .split();
    assert_eq!(
        s,
        uefi::Status::BUFFER_TOO_SMALL,
        "Unexpected error was returned."
    );

    Bytes::new(bytes.expect("The number of bytes was not returned."))
}
