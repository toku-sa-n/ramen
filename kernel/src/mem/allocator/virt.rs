use predefined_mmap::{BYTES_AVAILABLE_RAM, STACK_BASE};

use {
    crate::mem::paging,
    core::convert::TryFrom,
    os_units::NumOfPages,
    predefined_mmap::KERNEL_ADDR,
    x86_64::{
        structures::paging::{PageSize, Size4KiB},
        VirtAddr,
    },
};

pub(crate) fn search_free_addr_for_user(num_pages: NumOfPages<Size4KiB>) -> Option<VirtAddr> {
    let mut cnt = 0;
    let mut start = None;
    for addr in (0..KERNEL_ADDR.as_u64()).step_by(usize::try_from(Size4KiB::SIZE).unwrap()) {
        let addr = VirtAddr::new(addr as _);
        if available(addr) {
            if start.is_none() {
                start = Some(addr);
            }

            cnt += 1;

            if cnt >= num_pages.as_usize() {
                return start;
            }
        } else {
            cnt = 0;
            start = None;
        }
    }

    None
}

pub(crate) fn search_free_addr_for_kernel(num_pages: NumOfPages<Size4KiB>) -> Option<VirtAddr> {
    let mut cnt = 0;
    let mut start = None;
    for addr in (STACK_BASE.as_u64()..BYTES_AVAILABLE_RAM.as_usize().try_into().unwrap())
        .step_by(usize::try_from(Size4KiB::SIZE).unwrap())
    {
        let addr = VirtAddr::new(addr as _);
        if available(addr) {
            if start.is_none() {
                start = Some(addr);
            }

            cnt += 1;

            if cnt >= num_pages.as_usize() {
                return start;
            }
        } else {
            cnt = 0;
            start = None;
        }
    }

    None
}

fn available(addr: VirtAddr) -> bool {
    paging::translate_addr(addr).is_none() && !addr.is_null()
}
