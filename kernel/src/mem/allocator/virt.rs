use {
    crate::mem::paging::pml4,
    core::convert::TryFrom,
    os_units::NumOfPages,
    predefined_mmap::BYTES_AVAILABLE_RAM,
    x86_64::{
        structures::paging::{PageSize, Size4KiB},
        VirtAddr,
    },
};

pub(crate) fn search_free_addr(num_pages: NumOfPages<Size4KiB>) -> Option<VirtAddr> {
    let mut cnt = 0;
    let mut start = None;
    for addr in
        (0..BYTES_AVAILABLE_RAM.as_usize()).step_by(usize::try_from(Size4KiB::SIZE).unwrap())
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
    pml4::translate_addr(addr).is_none() && !addr.is_null()
}
