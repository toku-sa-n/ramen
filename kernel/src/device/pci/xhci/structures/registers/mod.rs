// SPDX-License-Identifier: GPL-3.0-or-later

pub mod capability;
pub mod doorbell;
pub mod extended_capability;
pub mod operational;
pub mod runtime;
pub mod usb_legacy_support_capability;

use capability::Capability;
use operational::Operational;
use runtime::Runtime;
use usb_legacy_support_capability::UsbLegacySupportCapability;
use x86_64::PhysAddr;

pub struct Registers {
    pub usb_legacy_support_capability: Option<UsbLegacySupportCapability>,
    pub capability: Capability,
    pub operational: Operational,
    pub runtime: Runtime,
    pub doorbell_array: doorbell::Array,
}
impl Registers {
    /// SAFETY: This method is unsafe because if `mmio_base` is not the valid MMIO base address,
    /// it can violate memory safety.
    pub unsafe fn new(mmio_base: PhysAddr) -> Self {
        let capability = Capability::new(mmio_base);
        let usb_legacy_support_capability = UsbLegacySupportCapability::new(mmio_base, &capability);
        let operational = Operational::new(mmio_base, &capability);
        let runtime = Runtime::new(mmio_base, capability.rts_off.read().get() as usize);
        let doorbell_array = doorbell::Array::new(mmio_base, capability.db_off.read().get());

        Self {
            usb_legacy_support_capability,
            capability,
            operational,
            runtime,
            doorbell_array,
        }
    }
}
