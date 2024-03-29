// SPDX-License-Identifier: GPL-3.0-or-later

use {
    boot_info::mem::MemoryDescriptor,
    core::{
        convert::TryInto,
        mem::size_of,
        ptr::{self, NonNull},
    },
    log::info,
    os_units::NumOfPages,
    static_assertions::const_assert,
    uefi::table::{boot, boot::MemoryType, Boot, SystemTable},
    x86_64::PhysAddr,
};

const_assert!(size_of::<MemoryDescriptor>() < size_of::<boot::MemoryDescriptor>());

/// # Panics
///
/// This function panics if it fails to allocate a memory for the memory map.
#[must_use]
pub fn boot_services(system_table: SystemTable<Boot>) -> boot_info::mem::Map {
    info!("Goodbye, boot services...");
    let memory_map_buf = system_table
        .boot_services()
        .allocate_pool(
            MemoryType::LOADER_DATA,
            system_table.boot_services().memory_map_size().map_size,
        )
        .expect("Failed to allocate memory for memory map");
    let memory_map_buf = NonNull::new(memory_map_buf).expect("`memory_map_buf` must not be Null.");
    let memory_map_buf: NonNull<MemoryDescriptor> = memory_map_buf.cast();

    let (_, mut descriptors_iter) = system_table.exit_boot_services();

    descriptors_iter.sort();

    let mut entries = descriptors_iter.entries();
    let num_descriptors = entries.len();
    let memory_map_buf = write_descriptors_on_buf(memory_map_buf, &mut entries);
    boot_info::mem::Map::new(memory_map_buf, num_descriptors)
}

fn write_descriptors_on_buf(
    buf: NonNull<MemoryDescriptor>,
    iter: &mut dyn ExactSizeIterator<Item = &boot::MemoryDescriptor>,
) -> NonNull<MemoryDescriptor> {
    for (index, descriptor) in iter.enumerate() {
        let boot_info_descriptor = uefi_descriptor_to_boot_info(descriptor);

        unsafe {
            ptr::write(
                buf.as_ptr().offset(index.try_into().unwrap()),
                boot_info_descriptor,
            );
        }
    }

    buf
}

fn uefi_descriptor_to_boot_info(d: &boot::MemoryDescriptor) -> MemoryDescriptor {
    // SAFETY: `d` contains the correct information as it is generated by `exit_boot_services`.
    unsafe {
        MemoryDescriptor::new(
            PhysAddr::new(d.phys_start),
            NumOfPages::new(d.page_count.try_into().unwrap()),
        )
    }
}
