// SPDX-License-Identifier: GPL-3.0-or-later

pub mod capability;
pub mod doorbell;
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
        let hc_capability_registers = Capability::new(mmio_base);
        let usb_legacy_support_capability =
            UsbLegacySupportCapability::new(mmio_base, &hc_capability_registers);
        let hc_operational = Operational::new(mmio_base, &hc_capability_registers);
        let runtime_base_registers = Runtime::new(
            mmio_base,
            hc_capability_registers.rts_off.read().get() as usize,
        );
        let doorbell_array =
            doorbell::Array::new(mmio_base, hc_capability_registers.db_off.read().get());

        Self {
            usb_legacy_support_capability,
            capability: hc_capability_registers,
            operational: hc_operational,
            runtime: runtime_base_registers,
            doorbell_array,
        }
    }
}
