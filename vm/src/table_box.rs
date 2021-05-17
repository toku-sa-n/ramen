use core::ops::Deref;
use x86_64::{structures::paging::PageTable, VirtAddr};

struct TableBox {
    virt: VirtAddr,
}
impl Deref for TableBox {
    type Target = PageTable;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.virt.as_ptr() }
    }
}
