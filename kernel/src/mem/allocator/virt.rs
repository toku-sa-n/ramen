use {
    crate::mem::paging,
    os_units::NumOfPages,
    x86_64::{
        structures::paging::{page::PageRange, Size4KiB},
        VirtAddr,
    },
};

pub(crate) fn search_free_addr_from(
    num_pages: NumOfPages<Size4KiB>,
    region: PageRange,
) -> Option<VirtAddr> {
    let mut cnt = 0;
    let mut start = None;
    for page in region {
        let addr = page.start_address();
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
