use {
    crate::mem::allocator::phys,
    conquer_once::spin::Lazy,
    predefined_mmap::RECUR_PML4_ADDR,
    spinning_top::Spinlock,
    x86_64::structures::paging::{
        mapper::{MapToError, MapperFlush},
        Mapper, Page, PageTableFlags, PhysFrame, RecursivePageTable, Size4KiB,
    },
};

pub(crate) static PML4: Lazy<Spinlock<RecursivePageTable<'_>>> = Lazy::new(|| unsafe {
    Spinlock::new(
        (RecursivePageTable::new(&mut *(RECUR_PML4_ADDR.as_mut_ptr())))
            .expect("PML4 has no recursive entry."),
    )
});

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
