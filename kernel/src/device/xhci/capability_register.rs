// SPDX-License-Identifier: GPL-3.0-or-later

use crate::{allocator::FRAME_MANAGER, device::pci::config::bar::Bar, mem::paging::pml4::PML4};
use common::constant::XHCI_CAPABILITY_REGISTER_ADDR;
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
        let page = Page::<Size4KiB>::containing_address(XHCI_CAPABILITY_REGISTER_ADDR);
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
                XHCI_CAPABILITY_REGISTER_ADDR
                    .as_mut_ptr::<u64>()
                    .offset(0x10),
            ) >> 16,
        ))
    }
}
