use uefi::prelude::{Boot, SystemTable};
use uefi::proto::media::file;
use uefi::proto::media::file::File;
use uefi::proto::media::file::FileAttribute;
use uefi::proto::media::file::FileMode;
use uefi::proto::media::fs;
use uefi::table::boot::{AllocateType, MemoryType};
use uefi::ResultExt;

struct KernelFileInfo {
    name: &'static str,
    start_address: usize,
    bytes: usize,
}

const BYTES_OF_PAGE: usize = 0x1000;

impl KernelFileInfo {
    const fn new(name: &'static str, start_address: usize, bytes: usize) -> Self {
        Self {
            name,
            start_address,
            bytes,
        }
    }

    fn get_filename(&self) -> &'static str {
        self.name
    }

    fn address(&self) -> usize {
        self.start_address
    }

    fn num_of_pages(&self) -> usize {
        (self.bytes + BYTES_OF_PAGE - 1) / BYTES_OF_PAGE
    }
}

// Using the size of binary as the memory consumption is useless because the size of .bss section
// is not included in the binary size. Using ELF file may improve effeciency as it might contain
// the size of memory comsuption.
const KERNEL_FILE: KernelFileInfo = KernelFileInfo::new("kernel.bin", 0x200000, 0x20000);

fn open_file(file: &KernelFileInfo, root_dir: &mut file::Directory) -> file::FileHandle {
    let file_handler = root_dir.open(file.get_filename(), FileMode::Read, FileAttribute::empty());

    // Use panic!, not expect_success() to print the detail of error.
    if let Err(e) = file_handler {
        panic!("Failed to open file: {} {:?}", file.get_filename(), e);
    }

    file_handler.unwrap().unwrap()
}

fn allocate_for_kernel_file(system_table: &SystemTable<Boot>, file: &KernelFileInfo) -> () {
    let status = system_table.boot_services().allocate_pages(
        AllocateType::Address(file.address()),
        MemoryType::LOADER_DATA,
        file.num_of_pages(),
    );

    // Use panic!, not expect_success() to know which allocation fails.
    if let Err(e) = status {
        panic!(
            "Failed to allocate memory for {}: {:?}",
            file.get_filename(),
            e
        );
    }

    // It is not necessary to return the address as it is fixed.
}

fn read_kernel_on_memory(file: &KernelFileInfo, handler: &mut file::RegularFile) -> () {
    // Reading should use while statement with the number of bytes which were actually read.
    // However, without while statement previous uefi implementation worked so this uefi
    // implementation also never use it.
    let status = handler.read(unsafe {
        core::slice::from_raw_parts_mut(
            file.address() as *mut u8,
            file.num_of_pages() * BYTES_OF_PAGE,
        )
    });

    // Use panic! to know which files causes an error.
    if let Err(e) = status {
        panic!("Failed to read {}: {:?}", file.get_filename(), e);
    }
}

fn open_kernel(
    system_table: &SystemTable<Boot>,
    file: &KernelFileInfo,
    root_dir: &mut file::Directory,
) -> () {
    let file_handler = open_file(file, root_dir);

    // Kernel file is a regular file, not a directory.
    // This `new` always succeeds.
    let mut file_handler = unsafe { file::RegularFile::new(file_handler) };
    allocate_for_kernel_file(system_table, file);
    read_kernel_on_memory(file, &mut file_handler);
}

pub fn place_kernel(system_table: &SystemTable<Boot>) -> () {
    let mut root_dir = open_root_dir(system_table);

    open_kernel(system_table, &KERNEL_FILE, &mut root_dir);
}

fn open_root_dir(system_table: &SystemTable<Boot>) -> file::Directory {
    let simple_file_system = system_table
        .boot_services()
        .locate_protocol::<fs::SimpleFileSystem>()
        .expect_success("Failed to prepare simple file system.");

    let simple_file_system = unsafe { &mut *simple_file_system.get() };

    simple_file_system
        .open_volume()
        .expect_success("Failed to open volume.")
}
