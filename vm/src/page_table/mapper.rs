use core::convert::TryInto;

use crate::frame;
use conquer_once::spin::Lazy;
use spinning_top::Spinlock;
use x86_64::{
    structures::paging::{
        Mapper, Page, PageSize, PageTableFlags, PhysFrame, RecursivePageTable, Size4KiB, Translate,
    },
    VirtAddr,
};

static MAPPER: Lazy<Spinlock<RecursivePageTable>> = Lazy::new(|| {
    let a = 0xff7f_bfdf_e000 as *mut _;
    let a = unsafe { &mut *a };
    let t = RecursivePageTable::new(a);
    let t = t.expect("The recursive paging is not enabled.");

    Spinlock::new(t)
});

fn map_frame(frame: PhysFrame, flags: PageTableFlags) -> VirtAddr {
    let v = find_unused_virt_addr();
    let p = Page::from_start_address(v);
    let p = p.expect("Address is not page-aligned.");

    map(p, frame, flags);
    v
}

fn map(page: Page, frame: PhysFrame, flags: PageTableFlags) {
    let m = MAPPER.try_lock();
    let mut m = m.expect("Failed to lock `MAPPER`");

    let r = unsafe { m.map_to(page, frame, flags, &mut frame::Allocator) };
    let flush = r.expect("Failed to map a page.");
    flush.flush();
}

fn unmap(page: Page) {
    let m = MAPPER.try_lock();
    let mut m = m.expect("Failed to lock `MAPPER`");

    let r = m.unmap(page);
    let (_, flush) = r.expect("Failed to unmap a page.");

    flush.flush();
}

fn find_unused_virt_addr() -> VirtAddr {
    let m = MAPPER.try_lock();
    let mut m = m.expect("Failed to lock `MAPPER`");

    for a in (Size4KiB::SIZE..!0).step_by(Size4KiB::SIZE.try_into().unwrap()) {
        let a = VirtAddr::new(a);

        if m.translate_addr(a).is_none() {
            return a;
        }
    }

    panic!("All pages are used.");
}
