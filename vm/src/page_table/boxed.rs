use crate::page_table::mapper;
use core::ops::{Deref, DerefMut};
use os_units::NumOfPages;
use x86_64::structures::paging::{Page, PageTable, PageTableFlags, PhysFrame};

pub(crate) struct Boxed {
    page: Page,
    frame: PhysFrame,
}
impl Boxed {
    fn new() -> Self {
        let phys = syscalls::allocate_phys_frame(NumOfPages::new(1));
        let frame = PhysFrame::from_start_address(phys);
        let frame = frame.expect("The physical address is not page-aligned.");

        let flags =
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;

        let virt = mapper::map_frame(frame, flags);
        let page = Page::from_start_address(virt);
        let page = page.expect("The virtual address is not page-aligned");

        Self { page, frame }
    }
}
impl Deref for Boxed {
    type Target = PageTable;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.page.start_address().as_ptr() }
    }
}
impl DerefMut for Boxed {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.page.start_address().as_mut_ptr() }
    }
}
