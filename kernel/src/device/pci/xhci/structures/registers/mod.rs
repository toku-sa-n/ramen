// SPDX-License-Identifier: GPL-3.0-or-later

pub mod capability;
pub mod doorbell;
pub mod extended_capability;
pub mod operational;
pub mod runtime;

use capability::Capability;
use operational::Operational;
use runtime::Runtime;
use x86_64::PhysAddr;

pub struct Registers {
    pub extended_capability: Option<extended_capability::List>,
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
        let extended_capability = extended_capability::List::new(mmio_base, &capability);
        let operational = Operational::new(mmio_base, &capability);
        let runtime = Runtime::new(mmio_base, capability.rts_off.read().get() as usize);
        let doorbell_array = doorbell::Array::new(mmio_base, capability.db_off.read().get());

        Self {
            extended_capability,
            capability,
            operational,
            runtime,
            doorbell_array,
        }
    }
}
