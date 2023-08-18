use elfloader::RelocationEntry;

use {
    super::paging,
    aligned_ptr::ptr,
    elfloader::{ElfBinary, ElfLoader, ElfLoaderErr, Flags, LoadableHeaders, ProgramHeader, VAddr},
    x86_64::{
        structures::paging::{
            mapper::{FlagUpdateError, MapToError},
            page::PageRange,
            Page, PageSize, PageTableFlags, Size4KiB,
        },
        VirtAddr,
    },
};

pub(crate) unsafe fn map_to_current_address_space(binary: &[u8]) -> Result<VirtAddr, ElfLoaderErr> {
    let elf = ElfBinary::new(binary)?;

    elf.load(&mut Loader)?;

    Ok(VirtAddr::new(elf.entry_point()))
}

struct Loader;
impl Loader {
    fn allocate_for_header(header: ProgramHeader<'_>) -> Result<(), MapToError<Size4KiB>> {
        let page_range = Self::page_range_from_header(header);

        let flags =
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;

        paging::map_range_to_unused_phys_range(page_range, flags)
    }

    fn page_range_from_header<S: PageSize>(header: ProgramHeader<'_>) -> PageRange<S> {
        Self::page_range_from_vaddr_and_len(
            header.virtual_addr(),
            header.mem_size().try_into().unwrap(),
        )
    }

    fn page_range_from_vaddr_and_len<S: PageSize>(base: VAddr, len: usize) -> PageRange<S> {
        let start = VirtAddr::new(base);

        let end = start + len;
        let end = end.align_up(S::SIZE);

        let start = Page::containing_address(start);
        let end = Page::containing_address(end);

        PageRange { start, end }
    }

    unsafe fn update_flags(page_range: PageRange, flags: Flags) -> Result<(), FlagUpdateError> {
        unsafe {
            paging::update_flags_for_range(page_range, Self::elf_flags_to_page_table_flags(flags))
        }
    }

    fn elf_flags_to_page_table_flags(flags: Flags) -> PageTableFlags {
        let mut page_table_flags = PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE;

        if flags.is_write() {
            page_table_flags |= PageTableFlags::WRITABLE;
        }

        if !flags.is_execute() {
            page_table_flags |= PageTableFlags::NO_EXECUTE;
        }

        page_table_flags
    }
}
impl ElfLoader for Loader {
    fn allocate(&mut self, load_headers: LoadableHeaders<'_, '_>) -> Result<(), ElfLoaderErr> {
        for header in load_headers {
            Self::allocate_for_header(header).expect("Failed to allocate memory.");
        }

        Ok(())
    }

    /// # Safety
    ///
    /// **This method is actually an unsafe one.**
    ///
    /// The caller must ensure that the addresses `base..(base + region.len())` must be allocated.
    fn load(&mut self, flags: Flags, base: VAddr, region: &[u8]) -> Result<(), ElfLoaderErr> {
        let base = VirtAddr::new(base);

        // SAFETY: The caller ensures that the addresses `base..(base+region.len())` are allocated.
        unsafe {
            ptr::copy_nonoverlapping(region.as_ptr(), base.as_mut_ptr(), region.len());
        }

        let page_range = Self::page_range_from_vaddr_and_len(base.as_u64(), region.len());

        unsafe {
            Self::update_flags(page_range, flags).expect("Failed to update flags.");
        }

        Ok(())
    }

    fn relocate(&mut self, _: RelocationEntry) -> Result<(), ElfLoaderErr> {
        todo!()
    }
}
