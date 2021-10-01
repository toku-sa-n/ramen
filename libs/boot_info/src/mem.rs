use core::{ptr::NonNull, slice};
use os_units::NumOfPages;
use x86_64::{structures::paging::Size4KiB, PhysAddr};

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Map {
    ptr: NonNull<MemoryDescriptor>,
    num_descriptors: usize,
}

impl Map {
    #[must_use]
    pub fn new(ptr: NonNull<MemoryDescriptor>, num_descriptors: usize) -> Self {
        Self {
            ptr,
            num_descriptors,
        }
    }

    #[must_use]
    pub fn as_mut_slice(&mut self) -> &mut [MemoryDescriptor] {
        unsafe { slice::from_raw_parts_mut(self.ptr.as_ptr(), self.num_descriptors) }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct MemoryDescriptor {
    pub start: PhysAddr,
    pub num_pages: NumOfPages<Size4KiB>,
}
impl MemoryDescriptor {
    /// # Safety
    ///
    /// The caller must ensure that the memory region of `num_pages` pages from the physical
    /// address `start` is available.
    #[must_use]
    pub const unsafe fn new(start: PhysAddr, num_pages: NumOfPages<Size4KiB>) -> Self {
        Self { start, num_pages }
    }
}
