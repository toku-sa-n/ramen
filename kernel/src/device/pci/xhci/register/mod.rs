// SPDX-License-Identifier: GPL-3.0-or-later

pub mod doorbell;
pub mod hc_capability;
pub mod hc_operational;
pub mod runtime_base_registers;
pub mod usb_legacy_support_capability;

use {
    hc_capability::{HCCapabilityRegisters, MaxNumOfErst, NumberOfDeviceSlots},
    hc_operational::HCOperational,
    runtime_base_registers::RuntimeBaseRegisters,
    usb_legacy_support_capability::UsbLegacySupportCapability,
    x86_64::PhysAddr,
};

pub struct Registers {
    pub usb_legacy_support_capability: Option<UsbLegacySupportCapability>,
    pub hc_capability: HCCapabilityRegisters,
    pub hc_operational: HCOperational,
    runtime_base_registers: RuntimeBaseRegisters,
    doorbell_array: doorbell::Array,
}
impl Registers {
    pub fn new(mmio_base: PhysAddr) -> Self {
        let hc_capability_registers = HCCapabilityRegisters::new(mmio_base);
        let usb_legacy_support_capability =
            UsbLegacySupportCapability::new(mmio_base, &hc_capability_registers);
        let hc_operational = HCOperational::new(mmio_base, &hc_capability_registers);
        let runtime_base_registers = RuntimeBaseRegisters::new(
            mmio_base,
            hc_capability_registers.offset_to_runtime_registers() as usize,
        );
        let doorbell_array = doorbell::Array::new(mmio_base, hc_capability_registers.db_off());

        Self {
            usb_legacy_support_capability,
            hc_capability: hc_capability_registers,
            hc_operational,
            runtime_base_registers,
            doorbell_array,
        }
    }

    pub fn max_num_of_erst(&self) -> MaxNumOfErst {
        self.hc_capability.max_num_of_erst()
    }

    pub fn num_of_device_slots(&self) -> NumberOfDeviceSlots {
        self.hc_capability.number_of_device_slots()
    }

    pub fn set_dcbaap(&mut self, addr: PhysAddr) {
        self.hc_operational.set_dcbaa_ptr(addr);
    }

    pub fn set_command_ring_pointer(&mut self, addr: PhysAddr) {
        self.hc_operational.set_command_ring_ptr(addr)
    }

    pub fn set_event_ring_segment_table_size(&mut self) {
        let max_num_of_erst = self.hc_capability.max_num_of_erst();
        self.runtime_base_registers
            .set_event_ring_segment_table_size(max_num_of_erst.into());
    }

    pub fn set_event_ring_segment_table_addr(&mut self, addr: PhysAddr) {
        self.runtime_base_registers
            .set_event_ring_segment_table_addr(addr)
    }

    pub fn set_event_ring_dequeue_pointer(&mut self, addr: PhysAddr) {
        self.runtime_base_registers.set_event_ring_dequeue_ptr(addr)
    }

    pub fn notify_to_hc(&mut self) {
        self.doorbell_array.notify_to_hc();
    }
}
