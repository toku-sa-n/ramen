use crate::page_table::mapper;
use core::ops::{Deref, DerefMut};
use os_units::NumOfPages;
use x86_64::{
    structures::paging::{PageTable, PageTableFlags, PhysFrame},
    PhysAddr, VirtAddr,
};

pub(crate) struct Boxed {
    virt: VirtAddr,
    phys: PhysAddr,
}
impl Boxed {
    fn new() -> Self {
        let phys = syscalls::allocate_phys_frame(NumOfPages::new(1));
        let frame = PhysFrame::from_start_address(phys);
        let frame = frame.expect("The physical address is not page-aligned.");

        let flags =
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;

        let virt = mapper::map_frame(frame, flags);

        Self { virt, phys }
    }
}
impl Deref for Boxed {
    type Target = PageTable;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.virt.as_ptr() }
    }
}
impl DerefMut for Boxed {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.virt.as_mut_ptr() }
    }
}
