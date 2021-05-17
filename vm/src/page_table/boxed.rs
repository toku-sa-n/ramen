use core::ops::{Deref, DerefMut};
use x86_64::{structures::paging::PageTable, VirtAddr};

struct Boxed {
    virt: VirtAddr,
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
