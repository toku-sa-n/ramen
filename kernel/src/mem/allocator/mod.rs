use {
    super::paging,
    core::convert::TryFrom,
    os_units::NumOfPages,
    x86_64::{
        structures::paging::{Page, PageSize, Size4KiB},
        PhysAddr, VirtAddr,
    },
};

pub(crate) mod acpi;
pub(crate) mod heap;
pub(crate) mod kpbox;
pub(crate) mod phys;
pub(crate) mod virt;

pub(crate) fn allocate_pages_for_user(num_of_pages: NumOfPages<Size4KiB>) -> Option<VirtAddr> {
    let phys_addr = allocate_phys(num_of_pages)?;

    let virt_addr = super::map_pages_for_user(phys_addr, num_of_pages.as_bytes());

    Some(virt_addr)
}

pub(crate) fn allocate_pages_for_kernel(num_of_pages: NumOfPages<Size4KiB>) -> Option<VirtAddr> {
    let phys_addr = allocate_phys(num_of_pages)?;

    let virt_addr = super::map_pages_for_kernel(phys_addr, num_of_pages.as_bytes());

    Some(virt_addr)
}

pub(crate) fn deallocate_pages(virt: VirtAddr, num_of_pages: NumOfPages<Size4KiB>) {
    deallocate_phys(virt);
    deallocate_virt(virt, num_of_pages);
}

fn allocate_phys(num_of_pages: NumOfPages<Size4KiB>) -> Option<PhysAddr> {
    phys::alloc(num_of_pages)
}

fn deallocate_phys(virt: VirtAddr) {
    let phys = paging::translate_addr(virt).unwrap();
    phys::free(phys);
}

fn deallocate_virt(virt: VirtAddr, num_of_pages: NumOfPages<Size4KiB>) {
    for i in 0..u64::try_from(num_of_pages.as_usize()).unwrap() {
        let page = Page::<Size4KiB>::from_start_address(virt + Size4KiB::SIZE * i).unwrap();

        paging::unmap(page).unwrap();
    }
}
