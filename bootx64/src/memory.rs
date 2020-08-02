use core::ptr;
use uefi::prelude::{Boot, SystemTable};
use uefi::table::boot;
use uefi::table::boot::MemoryType;

/// (*mut u8, usize): (address to memory map, the size of memory map)
pub fn generate_map(system_table: &SystemTable<Boot>) -> (*mut u8, usize) {
    // Using returned value itself causes bufer too small erorr.
    // Doubling should solve this.
    let memory_map_size = system_table.boot_services().memory_map_size() * 2;

    info!("memory_map_size: {}", memory_map_size);

    // The last unwrap is for `Completion` type. Because the first `expect` is for `Result` type,
    // it is not needed to change `unwrap` to `expect_succes`.
    let memory_map_buf = system_table
        .boot_services()
        .allocate_pool(MemoryType::LOADER_DATA, memory_map_size)
        .expect("Failed to allocate memory for memory map")
        .unwrap();

    system_table
        .boot_services()
        .memory_map(unsafe { core::slice::from_raw_parts_mut(memory_map_buf, memory_map_size) })
        .expect("Failed to get memory map")
        .unwrap();

    (memory_map_buf, memory_map_size)
}

pub fn init_paging(mem_map: &mut [boot::MemoryDescriptor]) -> () {
    remove_table_protection();

    map_kernel(mem_map);
    map_stack(mem_map);
    map_idt(mem_map);
    map_vram(mem_map);
}

fn remove_table_protection() -> () {
    unsafe {
        asm!(
            "mov rax, cr0
        and eax, 0xfffeffff
        mov cr0, rax"
        )
    }
}

fn map_kernel(mem_map: &mut [boot::MemoryDescriptor]) -> () {
    map_virt_to_phys(0xffff_ffff_8000_0000, 0x0020_0000, 512 * 1024, mem_map);
}

fn map_idt(mem_map: &mut [boot::MemoryDescriptor]) -> () {
    map_virt_to_phys(
        0xffff_ffff_8000_0000 + 512 * 1024,
        0x0020_0000 + 512 * 1024,
        4 * 1024,
        mem_map,
    );
}

fn map_stack(mem_map: &mut [boot::MemoryDescriptor]) -> () {
    map_virt_to_phys(
        0xffff_ffff_8000_0000 + 516 * 1024,
        0x0020_0000 + 4 * 1024 + 512 * 1024,
        128 * 1024,
        mem_map,
    );
}

fn map_vram(mem_map: &mut [boot::MemoryDescriptor]) -> () {
    unsafe {
        map_virt_to_phys(
            0xffff_ffff_8020_0000,
            ptr::read(0x0ff8 as *const u64) as usize,
            ptr::read(0x0ff2 as *const u8) as usize
                * ptr::read(0x0ff4 as *const u16) as usize
                * ptr::read(0x0ff6 as *const u16) as usize
                / 8,
            mem_map,
        );
        ptr::write(0x0ff8 as *mut u64, 0xffff_ffff_8020_0000u64);
    }
}
fn map_virt_to_phys(
    virt: usize,
    phys: usize,
    bytes: usize,
    mem_map: &mut [boot::MemoryDescriptor],
) -> () {
    let num_of_pages = bytes_to_pages(bytes);

    for i in 0..num_of_pages {
        virt_points_phys(virt + BYTES_OF_PAGE * i, phys + BYTES_OF_PAGE * i, mem_map);
    }
}

fn bytes_to_pages(bytes: usize) -> usize {
    (bytes + BYTES_OF_PAGE - 1) / BYTES_OF_PAGE
}

fn virt_points_phys(virt: usize, phys: usize, mem_map: &mut [boot::MemoryDescriptor]) -> () {
    virt_points_phys_recur(virt, phys, get_pml4_addr(), mem_map, TableType::Pml4);
}

fn get_pml4_addr() -> usize {
    let addr;
    unsafe {
        asm!("mov rax, cr3",out("rax") addr,options(nomem, preserves_flags, nostack));
    }

    addr
}

fn virt_points_phys_recur(
    virt: usize,
    phys: usize,
    table_addr: usize,
    mem_map: &mut [boot::MemoryDescriptor],
    table: TableType,
) -> () {
    let ptr_to_entry = ptr_to_entry(virt, table_addr, table);

    if let TableType::Pt = table {
        return unsafe { ptr::write(ptr_to_entry, phys | PAGE_EXISTS) };
    }

    let mut entry = unsafe { ptr::read(ptr_to_entry) };

    if !entry_exists(entry) {
        entry = create_table(mem_map) | PAGE_EXISTS;
        unsafe { ptr::write(ptr_to_entry, entry) }
    }

    virt_points_phys_recur(
        virt,
        phys,
        get_addr_from_table_entry(entry),
        mem_map,
        table.next_table().unwrap(), // `table` can't be `Pt`. This `unwrap` always succeeds.
    )
}

fn get_offset_of_entry(virt_addr: usize, table: TableType) -> usize {
    (virt_addr
        >> match table {
            TableType::Pml4 => 39,
            TableType::Pdpt => 30,
            TableType::Pd => 21,
            TableType::Pt => 12,
        }
        & 0x1ff)
        * TABLE_ENTRY_SIZE
}

fn ptr_to_entry(virt: usize, table_addr: usize, table: TableType) -> *mut usize {
    (table_addr + get_offset_of_entry(virt, table)) as *mut _
}

fn entry_exists(entry: usize) -> bool {
    entry & PAGE_EXISTS == 1
}

fn create_table(mem_map: &mut [boot::MemoryDescriptor]) -> usize {
    let addr = allocate_page_for_page_table(mem_map);
    unsafe { initialize_page_table(addr) }

    addr
}

fn get_addr_from_table_entry(entry: usize) -> usize {
    entry & 0xffff_ffff_ffff_f000
}

fn allocate_page_for_page_table(mem_map: &mut [boot::MemoryDescriptor]) -> usize {
    for descriptor in mem_map.iter_mut() {
        if descriptor.ty == MemoryType::CONVENTIONAL {
            let addr = descriptor.phys_start;
            descriptor.phys_start += BYTES_OF_PAGE as u64;
            descriptor.page_count -= 1;

            return addr as usize;
        }
    }

    // Shouldn't reach here.
    panic!("Failed to allocate memory for a page table.");
}

unsafe fn initialize_page_table(table_addr: usize) -> () {
    ptr::write_bytes(table_addr as *mut u8, 0, BYTES_OF_PAGE)
}

#[derive(Copy, Clone)]
enum TableType {
    Pml4,
    Pdpt,
    Pd,
    Pt,
}

impl TableType {
    fn next_table(&self) -> Option<TableType> {
        match self {
            TableType::Pt => None,
            TableType::Pd => Some(TableType::Pt),
            TableType::Pdpt => Some(TableType::Pd),
            TableType::Pml4 => Some(TableType::Pdpt),
        }
    }
}

const TABLE_ENTRY_SIZE: usize = 8;

const PAGE_EXISTS: usize = 1;
const BYTES_OF_PAGE: usize = 0x1000;
