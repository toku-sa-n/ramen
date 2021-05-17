use conquer_once::spin::Lazy;
use spinning_top::Spinlock;
use x86_64::structures::paging::RecursivePageTable;

static MAPPER: Lazy<Spinlock<RecursivePageTable>> = Lazy::new(|| {
    let a = 0xff7f_bfdf_e000 as *mut _;
    let a = unsafe { &mut *a };
    let t = RecursivePageTable::new(a);
    let t = t.expect("The recursive paging is not enabled.");

    Spinlock::new(t)
});
