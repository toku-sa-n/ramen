// SPDX-License-Identifier: GPL-3.0-or-later

use core::{convert::TryInto, ptr, ptr::NonNull, slice};
use uefi::{
    table::{boot, boot::MemoryType, Boot, SystemTable},
    Handle, ResultExt,
};

/// # Panics
///
/// This function panics if it fails to allocate a memory for the memory map.
#[must_use]
pub fn boot_services(image: Handle, system_table: SystemTable<Boot>) -> common::mem::Map {
    info!("Goodbye, boot services...");
    let memory_map_buf = system_table
        .boot_services()
        .allocate_pool(
            MemoryType::LOADER_DATA,
            system_table.boot_services().memory_map_size(),
        )
        .expect_success("Failed to allocate memory for memory map");
    let memory_map_buf = NonNull::new(memory_map_buf).expect("`memory_map_buf` must not be Null.");
    let memory_map_buf: NonNull<boot::MemoryDescriptor> = memory_map_buf.cast();

    let buf = allocate_buf_for_exiting(system_table.boot_services());
    let (_, mut descriptors_iter) = system_table
        .exit_boot_services(image, buf)
        .expect_success("Failed to exit boot services");

    let num_descriptors = descriptors_iter.len();
    let memory_map_buf = write_descriptors_on_buf(memory_map_buf, &mut descriptors_iter);
    common::mem::Map::new(memory_map_buf, num_descriptors)
}

fn allocate_buf_for_exiting(bs: &boot::BootServices) -> &'static mut [u8] {
    // Allocate extra spaces because of paddings.
    let sz = bs.memory_map_size() * 2;
    let buf_for_exiting = bs
        .allocate_pool(MemoryType::LOADER_DATA, sz)
        .expect_success("Failed to allocate memory to exit boot services");
    unsafe { slice::from_raw_parts_mut(buf_for_exiting, sz) }
}

fn write_descriptors_on_buf(
    buf: NonNull<boot::MemoryDescriptor>,
    iter: &mut dyn ExactSizeIterator<Item = &boot::MemoryDescriptor>,
) -> NonNull<boot::MemoryDescriptor> {
    for (index, descriptor) in iter.enumerate() {
        unsafe {
            ptr::write(buf.as_ptr().offset(index.try_into().unwrap()), *descriptor);
        }
    }

    buf
}
