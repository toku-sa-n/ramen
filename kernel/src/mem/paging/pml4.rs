use {
    crate::mem::allocator::phys,
    conquer_once::spin::Lazy,
    predefined_mmap::RECUR_PML4_ADDR,
    spinning_top::Spinlock,
    x86_64::{
        structures::paging::{
            mapper::{MapToError, MapperFlush, UnmapError},
            Mapper, Page, PageTableFlags, PhysFrame, RecursivePageTable, Size4KiB, Translate,
        },
        PhysAddr, VirtAddr,
    },
};

pub(crate) static PML4: Lazy<Spinlock<RecursivePageTable<'_>>> = Lazy::new(|| unsafe {
    Spinlock::new(
        (RecursivePageTable::new(&mut *(RECUR_PML4_ADDR.as_mut_ptr())))
            .expect("PML4 has no recursive entry."),
    )
});

/// # Safety
///
/// Refer to [`x86_64::structures::paging::Mapper`].
pub(crate) unsafe fn map_to(
    page: Page,
    frame: PhysFrame,
    flags: PageTableFlags,
) -> Result<(), MapToError<Size4KiB>> {
    // SAFETY: The caller must ensure the all safety requirements.
    unsafe {
        PML4.lock()
            .map_to(page, frame, flags, &mut *phys::allocator())
            .map(MapperFlush::flush)
    }
}

pub(crate) fn unmap(page: Page) -> Result<PhysFrame, UnmapError> {
    PML4.lock().unmap(page).map(|(frame, flush)| {
        flush.flush();
        frame
    })
}

pub(crate) fn translate_addr(a: VirtAddr) -> Option<PhysAddr> {
    PML4.lock().translate_addr(a)
}
