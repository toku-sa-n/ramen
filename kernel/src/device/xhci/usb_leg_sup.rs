// SPDX-License-Identifier: GPL-3.0-or-later

use {
    crate::{
        device::xhci::capability_register::XhciExtendedCapabilitiesPointer,
        mem::{
            allocator::{phys::FRAME_MANAGER, virt},
            paging::pml4::PML4,
        },
        x86_64::structures::paging::{Mapper, Page, PageSize, PageTableFlags, PhysFrame, Size4KiB},
    },
    core::ptr,
};

struct UsbLegacySupportCapabilityRegister {
    xecp: XhciExtendedCapabilitiesPointer,
}

impl UsbLegacySupportCapabilityRegister {
    fn get_hc_bios_owned_semaphore(&self) -> bool {
        self.edit(|ptr_to_raw_reg| {
            let raw_value: u32 = unsafe { ptr::read(ptr_to_raw_reg) };

            (raw_value >> 16) & 1 == 1
        })
    }

    fn get_hc_os_owned_semaphore() -> bool {
        todo!()
    }

    fn set_hc_os_owned_semaphore() {
        todo!()
    }

    fn edit<T, U>(&self, f: T) -> U
    where
        T: Fn(*mut u32) -> U,
    {
        virt::map_temporary(|addr| {
            let page = Page::<Size4KiB>::containing_address(addr);
            let frame = PhysFrame::containing_address(self.xecp.get());
            unsafe {
                PML4.lock()
                    .map_to(
                        page,
                        frame,
                        PageTableFlags::PRESENT,
                        &mut *FRAME_MANAGER.lock(),
                    )
                    .unwrap()
                    .flush();
            }

            f(addr.as_mut_ptr())
        })
    }
}
