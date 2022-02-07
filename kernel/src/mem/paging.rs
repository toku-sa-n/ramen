use {
    crate::mem::allocator::phys,
    conquer_once::spin::Lazy,
    predefined_mmap::RECUR_PML4_ADDR,
    spinning_top::Spinlock,
    x86_64::{
        structures::paging::{
            mapper::{FlagUpdateError, MapToError, MapperFlush, UnmapError},
            page::PageRange,
            FrameAllocator, Mapper, Page, PageTable, PageTableFlags, PhysFrame, RecursivePageTable,
            Size4KiB, Translate,
        },
        PhysAddr, VirtAddr,
    },
};

static PML4: Lazy<Spinlock<RecursivePageTable<'_>>> = Lazy::new(|| unsafe {
    Spinlock::new(
        (RecursivePageTable::new(&mut *(RECUR_PML4_ADDR.as_mut_ptr())))
            .expect("PML4 has no recursive entry."),
    )
});

pub(crate) fn mark_pages_as_unused() {
    let page_table = unsafe { &mut *(RECUR_PML4_ADDR.as_mut_ptr::<PageTable>()) };

    // Entry 510 and 511 are used by kernel.
    for i in 0..510 {
        page_table[i].set_unused();
    }
}

pub(crate) fn map_range_to_unused_phys_range(
    page_range: PageRange,
    flags: PageTableFlags,
) -> Result<(), MapToError<Size4KiB>> {
    for p in page_range {
        map_to_unused(p, flags)?;
    }

    Ok(())
}

pub(crate) unsafe fn update_flags_for_range(
    page_range: PageRange,
    flags: PageTableFlags,
) -> Result<(), FlagUpdateError> {
    for p in page_range {
        unsafe {
            update_flags(p, flags)?;
        }
    }

    Ok(())
}

pub(crate) fn map_to_unused(page: Page, flags: PageTableFlags) -> Result<(), MapToError<Size4KiB>> {
    let frame = phys::allocator()
        .allocate_frame()
        .ok_or(MapToError::FrameAllocationFailed)?;

    unsafe { map_to(page, frame, flags) }
}

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

pub(crate) unsafe fn update_flags(
    page: Page,
    flags: PageTableFlags,
) -> Result<(), FlagUpdateError> {
    unsafe {
        PML4.lock()
            .update_flags(page, flags)
            .map(MapperFlush::flush)
    }
}

pub(crate) fn level_4_table() -> PageTable {
    PML4.lock().level_4_table().clone()
}
