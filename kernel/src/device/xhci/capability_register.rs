// SPDX-License-Identifier: GPL-3.0-or-later

use crate::{
    device::pci::config::bar::Bar,
    mem::{
        allocator::{phys::FRAME_MANAGER, virt},
        paging::pml4::PML4,
    },
};
use core::ptr;
use x86_64::{
    structures::paging::{FrameAllocator, Mapper, Page, PageTableFlags, PhysFrame, Size4KiB},
    PhysAddr,
};

pub(super) struct CapabilityRegister {
    xecp: XhciExtendedCapabilitiesPointer,
}

impl CapabilityRegister {
    pub(super) fn fetch(bar: &Bar) -> Self {
        Self::map(bar);

        Self {
            xecp: unsafe { XhciExtendedCapabilitiesPointer::fetch() },
        }
    }

    fn map(bar: &Bar) {
        let page = virt::search_first_unused_page().unwrap();
        let frame = PhysFrame::containing_address(bar.base_addr());

        unsafe {
            PML4.lock()
                .map_to(
                    page,
                    frame,
                    PageTableFlags::PRESENT,
                    &mut *FRAME_MANAGER.lock(),
                )
                .unwrap()
                .flush()
        };
    }
}

struct XhciExtendedCapabilitiesPointer(PhysAddr);
impl XhciExtendedCapabilitiesPointer {
    /// Safety: `XHCI_CAPABILITY_REGISTER_ADDR` must point the base of MMID.
    unsafe fn fetch() -> Self {
        Self(PhysAddr::new(
            ptr::read(
                virt::search_first_unused_page()
                    .unwrap()
                    .start_address()
                    .as_mut_ptr::<u64>()
                    .offset(0x10),
            ) >> 16,
        ))
    }
}
