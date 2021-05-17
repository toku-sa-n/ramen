use crate::frame;
use conquer_once::spin::Lazy;
use spinning_top::Spinlock;
use x86_64::structures::paging::{Mapper, Page, PageTableFlags, PhysFrame, RecursivePageTable};

static MAPPER: Lazy<Spinlock<RecursivePageTable>> = Lazy::new(|| {
    let a = 0xff7f_bfdf_e000 as *mut _;
    let a = unsafe { &mut *a };
    let t = RecursivePageTable::new(a);
    let t = t.expect("The recursive paging is not enabled.");

    Spinlock::new(t)
});

fn map(page: Page, frame: PhysFrame, flags: PageTableFlags) {
    let m = MAPPER.try_lock();
    let mut m = m.expect("Failed to lock `MAPPER`");

    let r = unsafe { m.map_to(page, frame, flags, &mut frame::Allocator) };
    let flush = r.expect("Failed to map a page.");
    flush.flush();
}
